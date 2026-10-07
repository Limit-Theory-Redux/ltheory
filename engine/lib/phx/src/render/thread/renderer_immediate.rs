use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crossbeam::channel::unbounded;
use tracing::{error, info};

#[cfg(feature = "stats-server")]
use crate::render::StatsSink;
use crate::render::thread::{CommandExecutor, CommandReply, RendererData};
use crate::render::{
    BindEntry, BindGroupId, BlendMode, BlockLayout, BufferId, CmdPrimitiveType, CullFace,
    GpuHandle, ImmVertex, PassCommands, PipelineDesc, PipelineId, RenderPassDesc, RenderStats,
    RenderThreadError, ResourceId, SamplerCache, SamplerDesc, SamplerId, ShaderLayout,
    ShaderReloadResult, TexFilter, TexFormat, TexView, TexWrapMode, VertexFormat,
};
use crate::window::{PresentMode, WindowGlContext};

pub struct Renderer {
    /// Executes commands inline
    executor: CommandExecutor,
    /// Generic renderer data
    pub(crate) data: RendererData,
    // Optional sink receiving a per-frame snapshot for the stats dashboard.
    // Dashboard-only state sits behind the feature so normal game builds
    // carry none of the publishing path - mirrors `renderer_threaded.rs`.
    #[cfg(feature = "stats-server")]
    pub(super) stats_sink: Option<StatsSink>,
    /// Shared with the executor: enables per-category timing when the sink is
    /// attached (dashboard mode). Kept on the renderer so `attach_stats_sink`
    /// can flip it, same as the threaded backend.
    #[cfg(feature = "stats-server")]
    pub(super) category_timing: Arc<AtomicBool>,
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenderThreadError> {
        let ctx = context.make_current().map_err(|e| {
            error!("Failed to make GL context current: {e}");
            e
        })?;
        info!("GL context made current");

        // Always created (matches `renderer_threaded.rs::create_intern`) so
        // `CommandExecutor` behaves identically whether or not the
        // `stats-server` feature attaches a sink to flip it - only the
        // `Renderer`-side field that lets `attach_stats_sink` flip it later
        // is feature-gated.
        let category_timing = Arc::new(AtomicBool::new(false));
        let mut executor = CommandExecutor::new_with_timing(Some(ctx), category_timing.clone());
        executor.init_gl();

        info!("Renderer started (immediate mode)");

        // Unbounded: `ResourceHandle::drop` must never block or fail.
        let (destroy_tx, destroy_rx) = unbounded();

        let mut renderer = Self {
            executor,
            data: RendererData::new(destroy_tx, destroy_rx),
            #[cfg(feature = "stats-server")]
            stats_sink: None,
            #[cfg(feature = "stats-server")]
            category_timing,
        };
        for (id, desc) in SamplerCache::presets() {
            renderer.create_sampler(id, desc);
        }
        Ok(renderer)
    }

    /// The executor, after handing the open pass's recorded commands to it:
    /// every non-pass command goes through here, so old and new commands in
    /// one pass execute in order. // S6: remove
    fn ex(&mut self) -> &mut CommandExecutor {
        self.flush_pass_encoder();
        // A legacy command may have changed the program behind the pipeline
        // the batcher thinks is bound. // S6: remove
        self.data.pass.bound_pipeline = None;
        &mut self.executor
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
    /// Immediate mode has nothing to wait for: by the time `submit` returns,
    /// the command has already executed.
    pub(super) fn sync_intern(&mut self) -> bool {
        true
    }

    pub fn set_viewport_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.ex().cmd_set_viewport(x, y, width, height);
    }

    pub fn set_scissor_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.ex().cmd_set_scissor(x, y, width, height);
    }

    pub fn enable_scissor_intern(&mut self, enable: bool) {
        self.ex().cmd_enable_scissor(enable);
    }

    pub fn set_blend_mode_intern(&mut self, mode: BlendMode) {
        self.ex().cmd_set_blend_mode(mode);
    }

    pub fn set_cull_face_intern(&mut self, face: CullFace) {
        self.ex().cmd_set_cull_face(face);
    }

    pub fn set_depth_test_intern(&mut self, enable: bool) {
        self.ex().cmd_set_depth_test(enable);
    }

    pub fn set_depth_writable_intern(&mut self, enable: bool) {
        self.ex().cmd_set_depth_writable(enable);
    }

    pub fn set_wireframe_intern(&mut self, enable: bool) {
        self.ex().cmd_set_wireframe(enable);
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.ex().cmd_set_line_width(width);
    }

    pub fn set_point_size(&mut self, size: f32) {
        self.ex().cmd_set_point_size(size);
    }

    // === Shader Operations ===

    pub fn bind_shader_intern(&mut self, handle: GpuHandle) {
        self.ex().cmd_bind_shader(handle);
    }

    pub fn bind_shader_by_resource(&mut self, id: ResourceId, shader_key: Option<String>) {
        self.ex().cmd_bind_shader_by_resource(id, shader_key);
    }

    pub fn unbind_shader_intern(&mut self) {
        self.ex().cmd_unbind_shader();
    }

    pub fn set_uniform_int_intern(&mut self, location: i32, value: i32) {
        self.ex().cmd_set_uniform_int(location, value);
    }

    pub fn set_uniform_int2(&mut self, location: i32, value: [i32; 2]) {
        self.ex().cmd_set_uniform_int2(location, value);
    }

    pub fn set_uniform_int3(&mut self, location: i32, value: [i32; 3]) {
        self.ex().cmd_set_uniform_int3(location, value);
    }

    pub fn set_uniform_int4(&mut self, location: i32, value: [i32; 4]) {
        self.ex().cmd_set_uniform_int4(location, value);
    }

    pub fn set_uniform_float_intern(&mut self, location: i32, value: f32) {
        self.ex().cmd_set_uniform_float(location, value);
    }

    pub fn set_uniform_float2_intern(&mut self, location: i32, value: [f32; 2]) {
        self.ex().cmd_set_uniform_float2(location, value);
    }

    pub fn set_uniform_float3_intern(&mut self, location: i32, value: [f32; 3]) {
        self.ex().cmd_set_uniform_float3(location, value);
    }

    pub fn set_uniform_float4_intern(&mut self, location: i32, value: [f32; 4]) {
        self.ex().cmd_set_uniform_float4(location, value);
    }

    pub fn set_uniform_mat4(&mut self, location: i32, value: [f32; 16]) {
        self.ex().cmd_set_uniform_mat4(location, value);
    }

    // === Texture Operations ===

    pub fn bind_texture_2d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.ex().cmd_bind_texture_2d(slot, handle);
    }

    pub fn bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.ex().cmd_bind_texture_2d_by_resource(slot, id);
    }

    pub fn bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.ex().cmd_bind_texture_1d_by_resource(slot, id);
    }

    pub fn bind_texture_3d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.ex().cmd_bind_texture_3d(slot, handle);
    }

    pub fn bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.ex().cmd_bind_texture_3d_by_resource(slot, id);
    }

    pub fn bind_texture_cube_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.ex().cmd_bind_texture_cube(slot, handle);
    }

    pub fn bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.ex().cmd_bind_texture_cube_by_resource(slot, id);
    }

    pub fn unbind_texture_intern(&mut self, slot: u32) {
        self.ex().cmd_unbind_texture(slot);
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
        self.ex().cmd_update_texture_2d_data_by_resource(
            id,
            width,
            height,
            internal_format,
            pixel_format,
            data_format,
            data,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_texture_2d_rect(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.ex().cmd_update_texture_2d_rect(
            id,
            x,
            y,
            width,
            height,
            pixel_format,
            data_format,
            data,
        );
    }

    pub fn set_texture_2d_anisotropy_by_resource(&mut self, id: ResourceId, factor: f32) {
        self.ex()
            .cmd_set_texture_2d_anisotropy_by_resource(id, factor);
    }

    pub fn set_texture_2d_mip_range_by_resource(
        &mut self,
        id: ResourceId,
        min_level: i32,
        max_level: i32,
    ) {
        self.ex()
            .cmd_set_texture_2d_mip_range_by_resource(id, min_level, max_level);
    }

    pub fn set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        self.ex().cmd_set_texel_1d_by_resource(id, x, color);
    }

    pub fn set_texel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        self.ex().cmd_set_texel_2d_by_resource(id, x, y, color);
    }

    pub fn set_texture_mag_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.ex().cmd_set_texture_mag_filter_by_resource(id, filter);
    }

    pub fn set_texture_min_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.ex().cmd_set_texture_min_filter_by_resource(id, filter);
    }

    pub fn set_texture_wrap_mode_by_resource(&mut self, id: ResourceId, mode: TexWrapMode) {
        self.ex().cmd_set_texture_wrap_mode_by_resource(id, mode);
    }

    pub fn generate_mipmap_by_resource(&mut self, id: ResourceId) {
        self.ex().cmd_generate_mipmap_by_resource(id);
    }

    /// Copy `size` texels (width, height, layers) from the origin of `src`
    /// to the origin of `dst` (see `RenderCommand::CopyTexture`).
    pub fn copy_texture(&mut self, src: TexView, dst: TexView, size: [u32; 3]) {
        self.ex().cmd_copy_texture(&src, &dst, size);
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
        self.ex().cmd_update_texture_1d_data_by_resource(
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
        self.ex().cmd_update_texture_3d_data_by_resource(
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
        self.ex().cmd_update_texture_cube_face_data_by_resource(
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
        self.ex().cmd_copy_texture_2d_from_framebuffer_by_resource(
            id,
            internal_format,
            width,
            height,
        );
    }

    pub fn read_texture_1d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        self.ex()
            .cmd_read_texture_1d_data(id, pixel_format, data_format)
    }

    pub fn read_texture_2d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        self.ex()
            .cmd_read_texture_2d_data(id, pixel_format, data_format)
    }

    pub fn read_texture_3d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        self.ex()
            .cmd_read_texture_3d_data(id, pixel_format, data_format)
    }

    pub fn read_texture_cube_face_data(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        self.ex()
            .cmd_read_texture_cube_face_data(id, face, level, pixel_format, data_format)
    }

    pub fn sample_pixel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32) -> [u8; 4] {
        self.ex().cmd_sample_pixel_2d_by_resource(id, x, y)
    }

    pub fn read_framebuffer_pixels(&mut self, x: i32, y: i32, width: i32, height: i32) -> Vec<u8> {
        self.ex().cmd_read_framebuffer_pixels(x, y, width, height)
    }

    // === Render Passes ===

    pub fn begin_render_pass(&mut self, desc: Box<RenderPassDesc>) {
        self.ex().cmd_begin_render_pass(&desc);
    }

    pub fn end_render_pass(&mut self) {
        self.ex().cmd_end_render_pass();
    }

    // === Drawing Operations ===

    pub fn draw_mesh_intern(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.ex().cmd_draw_mesh(vao, index_count, primitive);
    }

    pub fn draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.ex()
            .cmd_draw_mesh_by_resource(id, index_count, primitive);
    }

    pub fn draw_immediate(&mut self, primitive: CmdPrimitiveType, vertices: Vec<ImmVertex>) {
        self.ex().cmd_draw_immediate(primitive, &vertices);
    }

    // === Resource Creation ===

    pub fn create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        layout: Arc<ShaderLayout>,
    ) -> Result<Vec<BlockLayout>, String> {
        self.ex()
            .cmd_create_shader(id, vertex_src, fragment_src, &layout)
    }

    // === Binding model objects ===

    pub fn create_pipeline(&mut self, id: PipelineId, desc: Box<PipelineDesc>) {
        self.ex().cmd_create_pipeline(id, &desc);
    }

    pub fn create_sampler(&mut self, id: SamplerId, desc: SamplerDesc) {
        self.ex().cmd_create_sampler(id, &desc);
    }

    pub fn create_bind_group_intern(
        &mut self,
        id: BindGroupId,
        shader: ResourceId,
        group: u8,
        entries: Box<[BindEntry]>,
    ) {
        self.ex().cmd_create_bind_group(id, shader, group, &entries);
    }

    pub fn destroy_bind_groups(&mut self, ids: Vec<BindGroupId>) {
        self.ex().cmd_destroy_bind_groups(&ids);
    }

    pub fn create_buffer(&mut self, id: BufferId, size: u32) {
        self.ex().cmd_create_buffer(id, size);
    }

    pub fn write_buffer(&mut self, id: BufferId, offset: u32, data: Vec<u8>) {
        self.ex().cmd_write_buffer(id, offset, &data);
    }

    /// Run the open pass's recorded commands (called by `flush_pass_encoder`;
    /// goes to the executor directly, without flushing again).
    pub(crate) fn send_pass_commands(&mut self, mut commands: Box<PassCommands>) {
        self.executor.cmd_pass_commands(&mut commands);
        // Immediate mode hands the uploaded ring memory back inline.
        for chunk in self.executor.take_returned_chunks() {
            self.data.recycle_chunk(chunk);
        }
    }

    /// Ring memory comes back inline in `send_pass_commands`; nothing queued.
    pub(crate) fn reclaim_chunks(&mut self) {}

    pub fn get_uniform_location_by_resource(&mut self, id: ResourceId, name: Arc<str>) -> i32 {
        self.ex().cmd_get_uniform_location_by_resource(id, name)
    }

    pub fn create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.ex().cmd_create_texture_1d(id, width, format, data);
    }

    pub fn create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.ex()
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
        self.ex()
            .cmd_create_texture_3d(id, width, height, depth, format, data);
    }

    pub fn create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        self.ex().cmd_create_texture_cube(id, size, format);
    }

    pub fn create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        self.ex()
            .cmd_create_mesh(id, vertices, indices, vertex_format);
    }

    // === Window Operations ===

    /// Blocking resize - immediate mode has nothing to block on, so this is
    /// the same as `try_resize`.
    pub fn resize_intern(&mut self, width: u32, height: u32) {
        self.ex().cmd_resize(width, height);
    }

    /// Always succeeds - see `try_submit`.
    pub fn try_resize(&mut self, width: u32, height: u32) -> bool {
        self.resize_intern(width, height);
        true
    }

    pub fn swap_buffers_intern(&mut self) {
        self.ex().cmd_swap_buffers();
    }

    /// Change vsync at runtime. Immediate mode has no thread to hop to, so
    /// this calls straight into the executor - see the threaded backend's
    /// `set_present_mode` for why the API is the same shape on both.
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.ex().cmd_set_present_mode(mode);
    }

    /// Block until every previously-submitted GL command has completed
    /// (`glFinish`).
    pub fn gl_finish(&mut self) {
        self.ex().cmd_flush();
    }

    /// Submit `DestroyResource` for every resource dropped since the last drain.
    fn drain_destroy_queue(&mut self) {
        // Collect first: the `destroy_rx` borrow has to end before `submit`
        // takes `&mut self`.
        let ids: Vec<_> = self.data.destroy_rx.try_iter().collect();

        self.ex().cmd_destroy_resource(&ids);
    }

    /// Immediate mode has no frame queue to pace against - just swap.
    pub fn end_frame_triple_buffered(&mut self) {
        self.pass_end_frame();
        self.drain_releases();
        self.drain_destroy_queue();
        self.ex().cmd_swap_buffers();
        // The next frame's ring slot (see the threaded backend).
        let slot = self.data.ring.slot();
        self.ex().cmd_begin_frame(slot);

        // Publish the combined snapshot to the dashboard sink (if attached)
        #[cfg(feature = "stats-server")]
        self.publish_stats_snapshot();
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
        self.executor.stats_snapshot().draw_calls_cumulative
    }

    /// Get total state changes since start
    pub fn get_state_changes(&mut self) -> u64 {
        self.executor.stats_snapshot().state_changes_cumulative
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
        self.executor.stats_snapshot().commands
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&mut self) -> u64 {
        self.executor.stats_snapshot().draw_calls
    }

    /// Always zero - immediate mode never blocks waiting on a render thread.
    pub fn get_main_thread_wait_us(&self) -> u64 {
        0
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&mut self) -> u64 {
        self.executor
            .stats_snapshot()
            .texture_binds_skipped_cumulative
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
                error: Some("Executor returned no shader reload result".to_string()),
                program: 0,
            },
        }
    }

    /// Immediate mode has nothing pending to poll for - `stop()` already
    /// returns the context synchronously.
    pub fn take_returned_context(&self) -> Option<WindowGlContext> {
        None
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
            data: RendererData::new(destroy_tx, destroy_rx),
            #[cfg(feature = "stats-server")]
            stats_sink: None,
            #[cfg(feature = "stats-server")]
            category_timing: Arc::new(AtomicBool::new(false)),
        }
    }
}
