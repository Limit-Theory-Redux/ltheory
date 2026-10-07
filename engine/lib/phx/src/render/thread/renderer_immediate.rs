use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crossbeam::channel::unbounded;
use tracing::{error, info};

#[cfg(feature = "stats-server")]
use crate::render::StatsSink;
use crate::render::thread::{CommandExecutor, RendererData};
use crate::render::{
    BindEntry, BindGroupId, BlockLayout, BufferId, PassCommands, PipelineDesc, PipelineId,
    ReadSource, ReadbackSlot, ReadbackTicket, RenderPassDesc, RenderStats, RenderThreadError,
    ResourceId, SamplerCache, SamplerDesc, SamplerId, ShaderLayout, TexDesc, TexFormat, TexRegion,
    TexView, VertexFormat,
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
    /// every non-pass command goes through here, so a resource write in the
    /// middle of a pass executes in order with the draws around it.
    fn ex(&mut self) -> &mut CommandExecutor {
        self.flush_pass_encoder();
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

    // === Shader Operations ===

    // === Texture Operations ===

    pub fn set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        self.ex().cmd_set_texel_1d_by_resource(id, x, color);
    }

    pub fn set_texel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        self.ex().cmd_set_texel_2d_by_resource(id, x, y, color);
    }

    /// Copy `size` texels (width, height, layers) from the origin of `src`
    /// to the origin of `dst` (see `RenderCommand::CopyTexture`).
    pub fn copy_texture(&mut self, src: TexView, dst: TexView, size: [u32; 3]) {
        self.ex().cmd_copy_texture(&src, &dst, size);
    }

    pub fn copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        format: TexFormat,
        width: i32,
        height: i32,
    ) {
        self.ex()
            .cmd_copy_texture_2d_from_framebuffer_by_resource(id, format, width, height);
    }

    /// Create a texture. `data` is level 0 in the texture's own `TexFormat`
    /// layout (see `convert_texels`).
    pub fn create_texture(&mut self, id: ResourceId, desc: &TexDesc, data: Option<Vec<u8>>) {
        self.ex().cmd_create_texture(id, desc, data);
    }

    /// Write `data`, in the texture's own format, to `region` of a texture.
    pub fn update_texture(&mut self, id: ResourceId, region: TexRegion, data: Vec<u8>) {
        self.ex().cmd_update_texture(id, &region, data);
    }

    /// Fill the mip levels below 0 of a texture from level 0.
    pub fn generate_mips(&mut self, id: ResourceId) {
        self.ex().cmd_generate_mips(id);
    }

    /// Read `region` of `src` as `format` and wait for it. Screenshots, tests
    /// and tools only. Empty if the read failed.
    pub fn read_texture_sync(
        &mut self,
        src: ReadSource,
        region: TexRegion,
        format: TexFormat,
    ) -> Vec<u8> {
        self.ex().cmd_read_texture_sync(src, &region, format)
    }

    /// Start reading `region` of `src` as `format` without waiting. The
    /// ticket is ready two or three frames later (see `ReadbackTicket`).
    pub fn read_texture_async(
        &mut self,
        src: ReadSource,
        region: TexRegion,
        format: TexFormat,
    ) -> ReadbackTicket {
        let slot = ReadbackSlot::new();
        self.ex()
            .cmd_readback_async(src, &region, format, slot.clone());
        ReadbackTicket::new(slot, &region)
    }

    // === Render Passes ===

    pub fn begin_render_pass(&mut self, desc: Box<RenderPassDesc>) {
        self.ex().cmd_begin_render_pass(&desc);
    }

    pub fn end_render_pass(&mut self) {
        self.ex().cmd_end_render_pass();
    }

    // === Drawing Operations ===

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
        let dropped: Vec<_> = self.data.destroy_rx.try_iter().collect();
        // Textures a live bind group still samples wait (see `TexturePins`).
        let ids = self.data.tex_pins.destroyable(dropped);

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

    /// `key=value` lines describing the backend.
    pub fn backend_info_intern(&self) -> String {
        format!("{}renderer_mode=immediate
", self.executor.gl_backend_info())
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
