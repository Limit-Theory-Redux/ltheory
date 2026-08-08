use std::sync::Arc;

use crossbeam::channel::{bounded, unbounded};
use tracing::{error, info};

use crate::render::thread::{CommandExecutor, CommandReply, RendererData, process_batch_intern};
use crate::render::{
    BlendMode, ClipManager, CmdPrimitiveType, CullFace, DrawState, GpuHandle, ImmVertex,
    PrimitiveBuilder, RenderStateIntern, RenderStats, RenderTargetStack, RenderThreadError,
    ResourceHandle, ResourceId, ShaderReloadResult, ShaderVarMap, TexFilter, TexFormat,
    TexWrapMode, VertexFormat, VpStack,
};
use crate::window::WindowGlContext;

pub struct Renderer {
    /// Executes commands inline on whichever thread calls `submit`.
    executor: CommandExecutor,
    /// Generic renderer data
    pub(crate) data: RendererData,
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenderThreadError> {
        let ctx = context.make_current().map_err(|e| {
            error!("Failed to make GL context current: {e}");
            e
        })?;
        info!("GL context made current");

        let mut executor = CommandExecutor::new(Some(ctx));
        executor.init_gl();

        info!("Renderer started (immediate mode)");

        // Unbounded: `ResourceHandle::drop` must never block or fail.
        let (destroy_tx, destroy_rx) = unbounded();

        Ok(Self {
            executor,
            data: RendererData {
                next_resource_id: 1,
                destroy_tx,
                destroy_rx,
                command_buffer: vec![],
                active_batch: None,
                viewport: VpStack::new(),
                render_target: RenderTargetStack::new(),
                clip_rect: ClipManager::new(),
                render_state: RenderStateIntern::new(),
                imm: PrimitiveBuilder::new(),
                draw_state: DrawState::new(),
                shader_vars: ShaderVarMap::new(),
                ao_shader: None,
                occlusion_shader: None,
                irmap_shader: None,
            },
        })
    }

    /// A `Renderer` with no GL context at all - every command becomes a
    /// no-op (see `CommandExecutor::has_gl_context`). Only for unit tests
    /// that exercise CPU-side logic (e.g. HmGui layout) and have no window
    /// to draw a real `WindowGlContext` from.
    #[cfg(test)]
    pub fn new_headless() -> Self {
        let (destroy_tx, destroy_rx) = unbounded();

        Self {
            executor: CommandExecutor::new(None),
            data: RendererData {
                next_resource_id: 1,
                destroy_tx,
                destroy_rx,
                command_buffer: vec![],
                active_batch: None,
                viewport: VpStack::new(),
                render_target: RenderTargetStack::new(),
                clip_rect: ClipManager::new(),
                render_state: RenderStateIntern::new(),
                imm: PrimitiveBuilder::new(),
                draw_state: DrawState::new(),
                shader_vars: ShaderVarMap::new(),
                ao_shader: None,
                occlusion_shader: None,
                irmap_shader: None,
            },
        }
    }

    pub fn stop(mut self) -> Option<WindowGlContext> {
        info!("Stopping renderer (immediate mode)");
        self.executor.cleanup()
    }
}

// =========================================================================
// Per-command API - one method per `RenderCommand` variant that any code
// outside `render::thread` needs. Each calls the matching
// `CommandExecutor::cmd_*` method directly, skipping `submit`/`execute`
// entirely - there is no channel to serialize the command onto, so
// building a `RenderCommand` here would just be wasted allocation.
// =========================================================================

// === State Management ===

impl Renderer {
    /// Begin a new frame
    pub(in crate::render::thread) fn begin_frame_intern(&mut self) {
        self.data.command_buffer.clear();
    }

    /// Run every buffered command (from the batch API) inline.
    pub(in crate::render::thread) fn flush_intern(&mut self) {
        for cmd in self.data.command_buffer.drain(..) {
            self.executor.execute(cmd);
        }
    }

    /// Immediate mode has nothing to wait for: by the time `submit` returns,
    /// the command has already executed.
    pub(in crate::render::thread) fn sync_intern(&mut self) -> bool {
        true
    }

    pub fn set_viewport_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.executor.cmd_set_viewport(x, y, width, height);
    }

    pub fn set_scissor_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.executor.cmd_set_scissor(x, y, width, height);
    }

    pub fn enable_scissor_intern(&mut self, enable: bool) {
        self.executor.cmd_enable_scissor(enable);
    }

    pub fn set_blend_mode_intern(&mut self, mode: BlendMode) {
        self.executor.cmd_set_blend_mode(mode);
    }

    pub fn set_cull_face_intern(&mut self, face: CullFace) {
        self.executor.cmd_set_cull_face(face);
    }

    pub fn set_depth_test_intern(&mut self, enable: bool) {
        self.executor.cmd_set_depth_test(enable);
    }

    pub fn set_depth_writable_intern(&mut self, enable: bool) {
        self.executor.cmd_set_depth_writable(enable);
    }

    pub fn set_wireframe_intern(&mut self, enable: bool) {
        self.executor.cmd_set_wireframe(enable);
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.executor.cmd_set_line_width(width);
    }

    pub fn set_point_size(&mut self, size: f32) {
        self.executor.cmd_set_point_size(size);
    }

    // === Shader Operations ===

    pub fn bind_shader_intern(&mut self, handle: GpuHandle) {
        self.executor.cmd_bind_shader(handle);
    }

    pub fn bind_shader_by_resource(&mut self, id: ResourceId, shader_key: Option<String>) {
        self.executor.cmd_bind_shader_by_resource(id, shader_key);
    }

    pub fn unbind_shader_intern(&mut self) {
        self.executor.cmd_unbind_shader();
    }

    pub fn set_uniform_int_intern(&mut self, location: i32, value: i32) {
        self.executor.cmd_set_uniform_int(location, value);
    }

    pub fn set_uniform_int2(&mut self, location: i32, value: [i32; 2]) {
        self.executor.cmd_set_uniform_int2(location, value);
    }

    pub fn set_uniform_int3(&mut self, location: i32, value: [i32; 3]) {
        self.executor.cmd_set_uniform_int3(location, value);
    }

    pub fn set_uniform_int4(&mut self, location: i32, value: [i32; 4]) {
        self.executor.cmd_set_uniform_int4(location, value);
    }

    pub fn set_uniform_float_intern(&mut self, location: i32, value: f32) {
        self.executor.cmd_set_uniform_float(location, value);
    }

    pub fn set_uniform_float2_intern(&mut self, location: i32, value: [f32; 2]) {
        self.executor.cmd_set_uniform_float2(location, value);
    }

    pub fn set_uniform_float3_intern(&mut self, location: i32, value: [f32; 3]) {
        self.executor.cmd_set_uniform_float3(location, value);
    }

    pub fn set_uniform_float4_intern(&mut self, location: i32, value: [f32; 4]) {
        self.executor.cmd_set_uniform_float4(location, value);
    }

    pub fn set_uniform_mat4(&mut self, location: i32, value: [f32; 16]) {
        self.executor.cmd_set_uniform_mat4(location, value);
    }

    // === Texture Operations ===

    pub fn bind_texture_2d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.executor.cmd_bind_texture_2d(slot, handle);
    }

    pub fn bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.executor.cmd_bind_texture_2d_by_resource(slot, id);
    }

    pub fn bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.executor.cmd_bind_texture_1d_by_resource(slot, id);
    }

    pub fn bind_texture_3d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.executor.cmd_bind_texture_3d(slot, handle);
    }

    pub fn bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.executor.cmd_bind_texture_3d_by_resource(slot, id);
    }

    pub fn bind_texture_cube_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.executor.cmd_bind_texture_cube(slot, handle);
    }

    pub fn bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.executor.cmd_bind_texture_cube_by_resource(slot, id);
    }

    pub fn unbind_texture_intern(&mut self, slot: u32) {
        self.executor.cmd_unbind_texture(slot);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_texture_2d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.executor.cmd_update_texture_2d_data_by_resource(
            id,
            width,
            height,
            internal_format,
            pixel_format,
            data_format,
            data,
        );
    }

    pub fn set_texture_2d_anisotropy_by_resource(&mut self, id: ResourceId, factor: f32) {
        self.executor
            .cmd_set_texture_2d_anisotropy_by_resource(id, factor);
    }

    pub fn set_texture_2d_mip_range_by_resource(
        &mut self,
        id: ResourceId,
        min_level: i32,
        max_level: i32,
    ) {
        self.executor
            .cmd_set_texture_2d_mip_range_by_resource(id, min_level, max_level);
    }

    pub fn set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        self.executor.cmd_set_texel_1d_by_resource(id, x, color);
    }

    pub fn set_texel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        self.executor.cmd_set_texel_2d_by_resource(id, x, y, color);
    }

    pub fn set_texture_mag_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.executor
            .cmd_set_texture_mag_filter_by_resource(id, filter);
    }

    pub fn set_texture_min_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.executor
            .cmd_set_texture_min_filter_by_resource(id, filter);
    }

    pub fn set_texture_wrap_mode_by_resource(&mut self, id: ResourceId, mode: TexWrapMode) {
        self.executor
            .cmd_set_texture_wrap_mode_by_resource(id, mode);
    }

    pub fn generate_mipmap_by_resource(&mut self, id: ResourceId) {
        self.executor.cmd_generate_mipmap_by_resource(id);
    }

    pub fn update_texture_1d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.executor.cmd_update_texture_1d_data_by_resource(
            id,
            width,
            internal_format,
            pixel_format,
            data_format,
            data,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_texture_3d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        height: i32,
        depth: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.executor.cmd_update_texture_3d_data_by_resource(
            id,
            width,
            height,
            depth,
            internal_format,
            pixel_format,
            data_format,
            data,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_texture_cube_face_data_by_resource(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        size: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.executor.cmd_update_texture_cube_face_data_by_resource(
            id,
            face,
            level,
            size,
            internal_format,
            pixel_format,
            data_format,
            data,
        );
    }

    pub fn copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        internal_format: i32,
        width: i32,
        height: i32,
    ) {
        self.executor
            .cmd_copy_texture_2d_from_framebuffer_by_resource(id, internal_format, width, height);
    }

    pub fn read_texture_1d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_read_texture_1d_data(id, pixel_format, data_format, tx);
        rx.recv().unwrap_or_default()
    }

    pub fn read_texture_2d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_read_texture_2d_data(id, pixel_format, data_format, tx);
        rx.recv().unwrap_or_default()
    }

    pub fn read_texture_3d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_read_texture_3d_data(id, pixel_format, data_format, tx);
        rx.recv().unwrap_or_default()
    }

    pub fn read_texture_cube_face_data(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.executor.cmd_read_texture_cube_face_data(
            id,
            face,
            level,
            pixel_format,
            data_format,
            tx,
        );
        rx.recv().unwrap_or_default()
    }

    pub fn sample_pixel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32) -> [u8; 4] {
        let (tx, rx) = bounded(1);
        self.executor.cmd_sample_pixel_2d_by_resource(id, x, y, tx);
        rx.recv().unwrap_or([0; 4])
    }

    pub fn read_framebuffer_pixels(&mut self, x: i32, y: i32, width: i32, height: i32) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_read_framebuffer_pixels(x, y, width, height, tx);
        rx.recv().unwrap_or_default()
    }

    // === Framebuffer Operations ===

    pub fn push_framebuffer(&mut self, _id: u64, _width: i32, _height: i32) {
        self.executor.cmd_push_framebuffer();
    }

    pub fn pop_framebuffer(&mut self) {
        self.executor.cmd_pop_framebuffer();
    }

    pub fn framebuffer_attach_texture_2d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        level: i32,
    ) {
        self.executor
            .cmd_framebuffer_attach_texture_2d_by_resource(attachment, id, level);
    }

    pub fn framebuffer_attach_texture_3d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        layer: i32,
        level: i32,
    ) {
        self.executor
            .cmd_framebuffer_attach_texture_3d_by_resource(attachment, id, layer, level);
    }

    pub fn framebuffer_attach_texture_cube_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        face: u32,
        level: i32,
    ) {
        self.executor
            .cmd_framebuffer_attach_texture_cube_by_resource(attachment, id, face, level);
    }

    pub fn bind_framebuffer_intern(&mut self, handle: GpuHandle) {
        self.executor.cmd_bind_framebuffer(handle);
    }

    pub fn bind_default_framebuffer_intern(&mut self) {
        self.executor.cmd_bind_default_framebuffer();
    }

    pub fn clear_intern(&mut self, color: Option<[f32; 4]>, depth: Option<f32>) {
        self.executor.cmd_clear(color, depth);
    }

    // === Drawing Operations ===

    pub fn draw_mesh_intern(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.executor.cmd_draw_mesh(vao, index_count, primitive);
    }

    pub fn draw_mesh_instanced_intern(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.executor
            .cmd_draw_mesh_instanced(vao, index_count, instance_count, primitive);
    }

    pub fn draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.executor
            .cmd_draw_mesh_by_resource(id, index_count, primitive);
    }

    pub fn draw_immediate(&mut self, primitive: CmdPrimitiveType, vertices: Vec<ImmVertex>) {
        self.executor.cmd_draw_immediate(primitive, &vertices);
    }

    // === Resource Creation ===

    pub fn create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
    ) -> Option<String> {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_create_shader(id, vertex_src, fragment_src, tx);
        rx.recv()
            .unwrap_or_else(|_| Some("Renderer channel closed".to_string()))
    }

    pub fn get_uniform_location_by_resource(&mut self, id: ResourceId, name: Arc<str>) -> i32 {
        let (tx, rx) = bounded(1);
        self.executor
            .cmd_get_uniform_location_by_resource(id, name, tx);
        rx.recv().unwrap_or(-1)
    }

    pub fn create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.executor.cmd_create_texture_1d(id, width, format, data);
    }

    pub fn create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.executor
            .cmd_create_texture_2d(id, width, height, format, data);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_texture_3d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        depth: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.executor
            .cmd_create_texture_3d(id, width, height, depth, format, data);
    }

    pub fn create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        self.executor.cmd_create_texture_cube(id, size, format);
    }

    pub fn create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        self.executor
            .cmd_create_mesh(id, vertices, indices, vertex_format);
    }

    // === Uniform Buffer Objects ===

    pub fn create_camera_ubo_intern(&mut self) {
        self.executor.cmd_create_camera_ubo();
    }

    pub fn update_camera_ubo_intern(&mut self, data: Box<[u8; 288]>) {
        self.executor.cmd_update_camera_ubo(&data);
    }

    pub fn create_material_ubo_intern(&mut self) {
        self.executor.cmd_create_material_ubo();
    }

    pub fn update_material_ubo_intern(&mut self, data: [u8; 32]) {
        self.executor.cmd_update_material_ubo(&data);
    }

    pub fn create_light_ubo_intern(&mut self) {
        self.executor.cmd_create_light_ubo();
    }

    pub fn update_light_ubo_intern(&mut self, data: [u8; 32]) {
        self.executor.cmd_update_light_ubo(&data);
    }

    // === Window Operations ===

    /// Blocking resize - immediate mode has nothing to block on, so this is
    /// the same as `try_resize`.
    pub fn resize_intern(&mut self, width: u32, height: u32) {
        self.executor.cmd_resize(width, height);
    }

    /// Always succeeds - see `try_submit`.
    pub fn try_resize(&mut self, width: u32, height: u32) -> bool {
        self.resize_intern(width, height);
        true
    }

    pub fn swap_buffers_intern(&mut self) {
        self.executor.cmd_swap_buffers();
    }

    /// Block until every previously-submitted GL command has completed
    /// (`glFinish`). Named to avoid colliding with `flush()`/`flush_intern`,
    /// which drains the CPU-side batch command buffer - an unrelated concept.
    pub fn gl_finish(&mut self) {
        self.executor.cmd_flush();
    }

    /// Mint a new GPU resource: a unique `ResourceId` bundled with the means
    /// to destroy it (see `ResourceHandle`). This is the only way to obtain
    /// either, so a resource can never exist without its destructor wired up.
    pub fn create_resource(&mut self) -> ResourceHandle {
        let id = ResourceId(self.data.next_resource_id);
        self.data.next_resource_id += 1;
        ResourceHandle::new(id, self.data.destroy_tx.clone())
    }

    /// Submit `DestroyResource` for every resource dropped since the last drain.
    fn drain_destroy_queue(&mut self) {
        // Collect first: the `destroy_rx` borrow has to end before `submit`
        // takes `&mut self`.
        let ids: Vec<ResourceId> = self.data.destroy_rx.try_iter().collect();

        self.executor.cmd_destroy_resource(&ids);
    }

    /// Immediate mode has no frame queue to pace against - just swap.
    pub fn end_frame_triple_buffered(&mut self) {
        self.drain_destroy_queue();
        self.executor.cmd_swap_buffers();
    }

    /// Always zero - there is no queue for a frame to be "in flight" on.
    pub fn get_frames_in_flight(&self) -> u64 {
        0
    }

    /// Always true while this `Renderer` exists: `stop()` consumes `self`,
    /// so there is no "stopped but still around" state to observe.
    pub fn is_running(&self) -> bool {
        true
    }

    /// Get current render stats snapshot
    pub fn get_stats(&mut self) -> RenderStats {
        self.executor.stats_snapshot()
    }

    /// Get total commands processed since start
    pub fn get_commands_processed(&mut self) -> u64 {
        self.executor.stats_snapshot().commands_processed
    }

    /// Get total draw calls since start
    pub fn get_draw_calls(&mut self) -> u64 {
        self.executor.stats_snapshot().draw_calls
    }

    /// Get total state changes since start
    pub fn get_state_changes(&mut self) -> u64 {
        self.executor.stats_snapshot().state_changes
    }

    /// Get total frames rendered
    pub fn get_frame_count(&mut self) -> u64 {
        self.executor.stats_snapshot().frame_count
    }

    /// Get last frame render time in microseconds
    pub fn get_last_frame_time_us(&mut self) -> u64 {
        self.executor.stats_snapshot().last_frame_time_us
    }

    /// Get commands processed in last frame
    pub fn get_commands_last_frame(&mut self) -> u64 {
        self.executor.stats_snapshot().commands_last_frame
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&mut self) -> u64 {
        self.executor.stats_snapshot().draw_calls_last_frame
    }

    /// Always zero - immediate mode never blocks waiting on a render thread.
    pub fn get_main_thread_wait_us(&self) -> u64 {
        0
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&mut self) -> u64 {
        self.executor.stats_snapshot().texture_binds_skipped
    }

    /// Reload a shader inline and return the result directly - no channel
    /// round-trip needed since the executor answers synchronously.
    pub fn reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> ShaderReloadResult {
        let reply = self
            .executor
            .cmd_reload_shader(shader_key, vertex_src, fragment_src);

        match reply {
            CommandReply::ShaderReload(result) => result,
            _ => ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Executor returned no shader reload result".to_string()),
                program: 0,
            },
        }
    }

    pub fn process_batch(&mut self) {
        process_batch_intern(&mut self.data.active_batch, &mut self.data.command_buffer);
    }

    /// Immediate mode has nothing pending to poll for - `stop()` already
    /// returns the context synchronously.
    pub fn take_returned_context(&self) -> Option<WindowGlContext> {
        None
    }
}
