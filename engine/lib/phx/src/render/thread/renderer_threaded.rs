use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TrySendError, bounded, unbounded};
use tracing::{error, info};

#[cfg(feature = "stats-server")]
use crate::render::StatsSink;
use crate::render::thread::RenderThread;
use crate::render::{
    BindEntry, BindGroupId, BlockLayout, BufferId, PassCommands, PipelineDesc, PipelineId,
    ReadSource, ReadbackSlot, ReadbackTicket, RenderCommand, RenderPassDesc, RenderStats,
    RenderThreadConfig, RenderThreadError, RendererData, ResourceId, ReturnedChunk, SamplerCache,
    SamplerDesc, SamplerId, ShaderLayout, TexDesc, TexFormat, TexRegion, TexView, VertexFormat,
};
use crate::window::{PresentMode, WgpuStartupBundle};

/// Maximum frames in flight for triple buffering
const MAX_FRAMES_IN_FLIGHT: u64 = 3;

pub struct Renderer {
    /// Send commands to the render thread
    command_tx: Sender<RenderCommand>,
    /// Receive fence completions from the render thread - only for
    /// `sync_intern`'s blocking round-trips. Frame-pacing fences travel on
    /// their own `pacing_fence_rx` instead (see `RenderCommand::PacingFence`).
    fence_rx: Receiver<u64>,
    /// Receive frame-pacing fence completions, kept separate from `fence_rx`
    /// so `end_frame_triple_buffered` can never consume a fence meant for a
    /// concurrently-blocked `sync_intern` call, or vice versa.
    pacing_fence_rx: Receiver<u64>,
    /// Ring memory the executor has uploaded, coming back for reuse.
    chunk_return_rx: Receiver<ReturnedChunk>,
    /// Next fence ID to use
    next_fence_id: AtomicU64,
    /// Number of frames currently in flight (submitted but not rendered)
    frames_in_flight: AtomicU64,
    /// Whether the render thread is running
    running: Arc<AtomicBool>,
    /// Thread handle for joining; taken by `Drop`
    thread_handle: Option<JoinHandle<()>>,
    /// Receives a stats snapshot from the render thread once per frame
    stats_rx: Receiver<RenderStats>,
    /// Most recent snapshot received over `stats_rx`
    last_stats: RenderStats,
    /// Time spent blocked in `end_frame_triple_buffered`, in microseconds -
    /// purely a main-thread measurement, the executor has no part in it
    pub(super) main_thread_wait_us: u64,
    /// Time spent blocked in `submit()` because the command channel was full,
    /// accumulated over the current frame (microseconds). Complements
    /// `main_thread_wait_us`: this catches mid-frame producer stalls that the
    /// end-of-frame measurement misses.
    pub(super) send_blocked_us: u64,
    /// Number of `submit()` calls that blocked on a full channel this frame
    pub(super) send_block_count: u64,
    /// Highest command-channel occupancy observed this frame
    pub(super) channel_high_water: u64,
    // Optional sink receiving a per-frame snapshot for the stats dashboard.
    // Dashboard-only state sits behind the feature so normal game builds
    // carry none of the publishing path.
    #[cfg(feature = "stats-server")]
    pub(super) stats_sink: Option<StatsSink>,
    /// Backend description (`key=value` lines), empty for a headless renderer.
    backend_info: String,
    /// Generic renderer data
    pub(crate) data: RendererData,
}

impl Renderer {
    /// Start the render thread: the surface bundle (device, queue, surface)
    /// moves to it, where `WgpuCommandExecutor` executes the commands.
    pub fn start(bundle: WgpuStartupBundle) -> Result<Self, RenderThreadError> {
        Self::create_intern(Some(bundle))
    }

    fn create_intern(bundle: Option<WgpuStartupBundle>) -> Result<Self, RenderThreadError> {
        let config = RenderThreadConfig::default();
        // Use bounded channel for backpressure - SwapBuffers will block to sync with render thread
        let (command_tx, command_rx) = bounded(config.command_buffer_size);
        let (fence_tx, fence_rx) = bounded(config.fence_buffer_size);
        let (pacing_fence_tx, pacing_fence_rx) = bounded(config.fence_buffer_size);
        // Unbounded: the executor must never block returning ring memory.
        let (chunk_return_tx, chunk_return_rx) = unbounded();
        let (stats_tx, stats_rx) = bounded(1); // Only the latest snapshot matters
        // Unbounded: `ResourceHandle::drop` must never block or fail.
        let (destroy_tx, destroy_rx) = unbounded();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let backend_info = bundle
            .as_ref()
            .map(|b| super::render_thread::wgpu_backend_info(&b.adapter, &b.surface_config))
            .unwrap_or_default();

        let thread_handle =
            thread::Builder::new()
                .name("RenderThread".into())
                .spawn(move || {
                    RenderThread::new(
                        command_rx,
                        fence_tx,
                        pacing_fence_tx,
                        stats_tx,
                        chunk_return_tx,
                        running_clone,
                        bundle,
                    )
                    .run();
                })?;

        info!("Render thread spawned");

        let mut renderer = Self {
            command_tx,
            fence_rx,
            pacing_fence_rx,
            chunk_return_rx,
            next_fence_id: AtomicU64::new(1),
            frames_in_flight: AtomicU64::new(0),
            running,
            thread_handle: Some(thread_handle),
            stats_rx,
            last_stats: RenderStats::default(),
            main_thread_wait_us: 0,
            send_blocked_us: 0,
            send_block_count: 0,
            channel_high_water: 0,
            #[cfg(feature = "stats-server")]
            stats_sink: None,
            backend_info,
            data: RendererData::new(destroy_tx, destroy_rx),
        };
        for (id, desc) in SamplerCache::presets() {
            renderer.create_sampler(id, desc);
        }
        Ok(renderer)
    }

    /// Submit a non-pass command to the render thread. Anything recorded in
    /// the open pass goes out first, so old and new commands in one pass
    /// execute in order.
    fn submit(&mut self, cmd: RenderCommand) {
        self.flush_pass_encoder();
        self.send(cmd);
    }

    /// Send a command to the render thread without flushing the pass encoder.
    fn send(&mut self, cmd: RenderCommand) {
        if self.running.load(Ordering::Relaxed) {
            // Fast path: non-blocking try_send. Only when the bounded channel
            // is full do we fall back to a blocking send — and only then do we
            // time the stall, so `send_blocked_us` measures real
            // mid-frame producer blocking (the thing `main_thread_wait_us`
            // misses).
            match self.command_tx.try_send(cmd) {
                Ok(()) => {
                    // Track channel occupancy high-water mark each frame
                    let depth = self.command_tx.len() as u64;
                    if depth > self.channel_high_water {
                        self.channel_high_water = depth;
                    }
                }
                Err(TrySendError::Full(cmd)) => {
                    let depth = self.command_tx.len() as u64;
                    if depth > self.channel_high_water {
                        self.channel_high_water = depth;
                    }
                    let start = std::time::Instant::now();
                    if let Err(e) = self.command_tx.send(cmd) {
                        error!("Failed to send render command: {e:?}");
                    }
                    let blocked_us = start.elapsed().as_micros() as u64;
                    self.send_blocked_us += blocked_us;
                    self.send_block_count += 1;
                }
                Err(TrySendError::Disconnected(_)) => {
                    error!("Failed to send render command: channel disconnected");
                }
            }
        }
    }

    /// Submit a command to the render thread without blocking.
    /// Returns true if the command was sent, false if the channel was full.
    /// Use this for commands that can be safely dropped (like resize events).
    fn try_submit(&mut self, cmd: RenderCommand) -> bool {
        if self.running.load(Ordering::Relaxed) {
            match self.command_tx.try_send(cmd) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => {
                    // Channel full, command dropped (will be retried next frame)
                    false
                }
                Err(TrySendError::Disconnected(_)) => {
                    error!("Render thread disconnected");
                    false
                }
            }
        } else {
            false
        }
    }

    /// Submit `DestroyResource` for every resource dropped since the last drain.
    fn drain_destroy_queue(&mut self) {
        // Collect first: the `destroy_rx` borrow has to end before `submit`
        // takes `&mut self`.
        let dropped: Vec<_> = self.data.destroy_rx.try_iter().collect();
        // Textures a live bind group still samples wait (see `TexturePins`).
        let ids = self.data.tex_pins.destroyable(dropped);

        self.submit(RenderCommand::DestroyResources { ids });
    }

    /// End the current frame with triple buffering.
    /// Submits SwapBuffers and fence, blocks only if MAX_FRAMES_IN_FLIGHT are queued.
    /// Uses fence channel for proper synchronization when throttling is needed.
    pub fn end_frame_triple_buffered(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        self.pass_end_frame();
        self.drain_releases();
        self.drain_destroy_queue();

        // Track ALL time spent in this function (includes channel blocking)
        let frame_end_start = std::time::Instant::now();

        // Drain completed fences (non-blocking) to update in-flight count
        while self.pacing_fence_rx.try_recv().is_ok() {
            self.frames_in_flight.fetch_sub(1, Ordering::Relaxed);
        }

        // If at limit, block waiting for one frame to complete
        while self.frames_in_flight.load(Ordering::Relaxed) >= MAX_FRAMES_IN_FLIGHT {
            match self.pacing_fence_rx.recv() {
                Ok(_) => {
                    self.frames_in_flight.fetch_sub(1, Ordering::Relaxed);
                }
                Err(_) => return, // Channel closed
            }
        }

        // Submit SwapBuffers + fence to track this frame
        // Note: submit() can also block if command channel is full!
        self.submit(RenderCommand::SwapBuffers);
        let fence_id = self.next_fence_id.fetch_add(1, Ordering::Relaxed);
        self.submit(RenderCommand::PacingFence { fence_id });
        self.frames_in_flight.fetch_add(1, Ordering::Relaxed);
        // The next frame's ring slot: the executor waits for the GPU to be
        // done with that slot's previous frame before its buffers are reused.
        let slot = self.data.ring.slot();
        self.submit(RenderCommand::BeginFrame { slot });

        // Store total time spent in frame end (all blocking)
        self.main_thread_wait_us = frame_end_start.elapsed().as_micros() as u64;

        // Publish the combined snapshot to the dashboard sink (if attached)
        #[cfg(feature = "stats-server")]
        self.publish_stats_snapshot();

        // Reset per-frame producer counters for the next frame
        self.send_blocked_us = 0;
        self.send_block_count = 0;
        self.channel_high_water = 0;
    }

    /// Get current frames in flight count
    pub fn get_frames_in_flight(&self) -> u64 {
        self.frames_in_flight.load(Ordering::Relaxed)
    }

    /// Check if the render thread is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Drain the stats channel, keeping only the most recent snapshot -
    /// there is never a reason to look at a stale one when a newer one is
    /// waiting.
    fn refresh_stats(&mut self) {
        while let Ok(stats) = self.stats_rx.try_recv() {
            self.last_stats = stats;
        }
    }

    /// `key=value` lines describing the backend (empty without a device).
    pub fn backend_info_intern(&self) -> String {
        self.backend_info.clone()
    }

    /// Get current render stats snapshot
    pub fn get_stats(&mut self) -> RenderStats {
        self.refresh_stats();
        self.last_stats.clone()
    }

    /// Get total commands processed since start
    pub fn get_commands_processed(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.commands_processed
    }

    /// Get total draw calls since start
    pub fn get_draw_calls(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.draw_calls_cumulative
    }

    /// Get total state changes since start
    pub fn get_state_changes(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.state_changes_cumulative
    }

    /// Get total frames rendered
    pub fn get_frame_count(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.frame_count
    }

    /// Get last frame render time in microseconds
    pub fn get_last_frame_time_us(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.last_frame_time_us
    }

    /// Get commands processed in last frame
    pub fn get_commands_last_frame(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.commands
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.draw_calls
    }

    /// Get main thread wait time in microseconds (time spent waiting for render thread)
    pub fn get_main_thread_wait_us(&self) -> u64 {
        self.main_thread_wait_us
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.texture_binds_skipped_cumulative
    }
}

impl Drop for Renderer {
    /// Stop the render thread and wait for it. The executor owns the wgpu
    /// surface, which was created from the window's raw handles: it has to be
    /// gone before `Engine` drops the window (see `create_surface_bundle`).
    fn drop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            info!("Requesting render thread shutdown");
            self.submit(RenderCommand::Shutdown);
            self.running.store(false, Ordering::Relaxed);
        }
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Render thread panicked: {e:?}");
            }
        }
    }
}

// =========================================================================
// Per-command API - one method per `RenderCommand` variant that any code
// outside `render::thread` needs. Each constructs the matching
// `RenderCommand` and calls `submit`/`try_submit`.
// =========================================================================

// === State Management ===

impl Renderer {
    /// Synchronize with the render thread (wait for all commands to complete)
    pub(super) fn sync_intern(&mut self) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        let fence_id = self.next_fence_id.fetch_add(1, Ordering::Relaxed);
        self.submit(RenderCommand::Fence { fence_id });

        // Wait for the fence to be signaled
        loop {
            match self.fence_rx.recv() {
                Ok(id) if id == fence_id => return true,
                Ok(_) => continue,      // Not our fence, keep waiting
                Err(_) => return false, // Channel closed
            }
        }
    }

    // === Shader Operations ===

    // === Texture Operations ===

    /// Create a texture. `data` is level 0 in the texture's own `TexFormat`
    /// layout (see `convert_texels`).
    pub fn create_texture(&mut self, id: ResourceId, desc: &TexDesc, data: Option<Vec<u8>>) {
        self.submit(RenderCommand::CreateTexture {
            id,
            desc: Box::new(*desc),
            data,
        });
    }

    /// Write `data`, in the texture's own format, to `region` of a texture.
    pub fn update_texture(&mut self, id: ResourceId, region: TexRegion, data: Vec<u8>) {
        self.submit(RenderCommand::UpdateTexture { id, region, data });
    }

    /// Fill the mip levels below 0 of a texture from level 0.
    pub fn generate_mips(&mut self, id: ResourceId) {
        self.submit(RenderCommand::GenerateMips { id });
    }

    pub fn set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        self.submit(RenderCommand::SetTexel1DByResource { id, x, color });
    }

    pub fn set_texel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        self.submit(RenderCommand::SetTexel2DByResource { id, x, y, color });
    }

    /// Copy `size` texels (width, height, layers) from the origin of `src`
    /// to the origin of `dst` (see `RenderCommand::CopyTexture`).
    pub fn copy_texture(&mut self, src: TexView, dst: TexView, size: [u32; 3]) {
        self.submit(RenderCommand::CopyTexture { src, dst, size });
    }

    pub fn copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        format: TexFormat,
        width: i32,
        height: i32,
    ) {
        self.submit(RenderCommand::CopyTexture2DFromFramebufferByResource {
            id,
            format,
            width,
            height,
        });
    }

    /// Read `region` of `src` as `format` and wait for it (see
    /// `RenderCommand::ReadTextureSync`). Screenshots, tests and tools only:
    /// it stalls the main thread until the render thread and the GPU got
    /// there. Empty if the read failed.
    pub fn read_texture_sync(
        &mut self,
        src: ReadSource,
        region: TexRegion,
        format: TexFormat,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::ReadTextureSync {
            src,
            region,
            format,
            reply_tx: tx,
        });
        rx.recv().unwrap_or_default()
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
        self.submit(RenderCommand::ReadbackAsync {
            src,
            region,
            format,
            slot: slot.clone(),
        });
        ReadbackTicket::new(slot, &region)
    }

    // === Render Passes ===

    pub fn begin_render_pass(&mut self, desc: Box<RenderPassDesc>) {
        self.submit(RenderCommand::BeginRenderPass(desc));
    }

    pub fn end_render_pass(&mut self) {
        self.submit(RenderCommand::EndRenderPass);
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
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::CreateShader {
            id,
            vertex_src,
            fragment_src,
            layout,
            reply_tx: tx,
        });
        rx.recv()
            .unwrap_or_else(|_| Err("Renderer channel closed".to_string()))
    }

    // === Binding model objects ===

    pub fn create_pipeline(&mut self, id: PipelineId, desc: Box<PipelineDesc>) {
        self.submit(RenderCommand::CreatePipeline { id, desc });
    }

    pub fn create_sampler(&mut self, id: SamplerId, desc: SamplerDesc) {
        self.submit(RenderCommand::CreateSampler { id, desc });
    }

    pub fn create_bind_group_intern(
        &mut self,
        id: BindGroupId,
        shader: ResourceId,
        group: u8,
        entries: Box<[BindEntry]>,
    ) {
        self.submit(RenderCommand::CreateBindGroup {
            id,
            shader,
            group,
            entries,
        });
    }

    pub fn destroy_bind_groups(&mut self, ids: Vec<BindGroupId>) {
        self.submit(RenderCommand::DestroyBindGroups { ids });
    }

    pub fn create_buffer(&mut self, id: BufferId, size: u32) {
        self.submit(RenderCommand::CreateBuffer { id, size });
    }

    pub fn write_buffer(&mut self, id: BufferId, offset: u32, data: Vec<u8>) {
        self.submit(RenderCommand::WriteBuffer { id, offset, data });
    }

    /// Take back the ring memory the render thread has finished uploading.
    pub(crate) fn reclaim_chunks(&mut self) {
        while let Ok(chunk) = self.chunk_return_rx.try_recv() {
            self.data.recycle_chunk(chunk);
        }
    }

    /// Hand the open pass's recorded commands to the render thread (called by
    /// `flush_pass_encoder`, so it must not flush again).
    pub(crate) fn send_pass_commands(&mut self, commands: Box<PassCommands>) {
        self.send(RenderCommand::PassCommands(commands));
    }

    pub fn create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        self.submit(RenderCommand::CreateMesh {
            id,
            vertices,
            indices,
            vertex_format,
        });
    }

    // === Window Operations ===

    /// Blocking resize - waits for the command to be queued (see `submit`).
    pub fn resize_intern(&mut self, width: u32, height: u32) {
        self.submit(RenderCommand::Resize { width, height });
    }

    /// Non-blocking resize - drops the command if the channel is full
    /// instead of blocking (safe: a later `Resized` event supersedes it).
    pub fn try_resize(&mut self, width: u32, height: u32) -> bool {
        self.try_submit(RenderCommand::Resize { width, height })
    }

    pub fn swap_buffers_intern(&mut self) {
        self.submit(RenderCommand::SwapBuffers);
    }

    /// Change vsync at runtime. Blocking `submit`, not `try_submit`:
    /// `Engine::changed_window` only calls this on an actual delta, so a
    /// dropped command would leave the requested mode permanently unapplied
    /// instead of just being retried next frame like a dropped resize is.
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.submit(RenderCommand::SetPresentMode { mode });
    }

    /// Block the caller until the GPU has finished everything recorded so
    /// far: `Flush` submits it and makes the render thread wait for the GPU,
    /// and the fence behind it comes back once that wait is over. `false` if
    /// the render thread is gone. Not for frame rendering (it stalls the
    /// pipeline); for bounding offline GPU work and timing it.
    pub fn wait_for_gpu(&mut self) -> bool {
        self.submit(RenderCommand::Flush);
        self.sync_intern()
    }

    /// A `Renderer` without a GPU device - every command becomes a no-op.
    /// Only for unit tests that exercise CPU-side logic (e.g. HmGui layout)
    /// and have no window to create a surface from.
    #[cfg(test)]
    pub fn new_headless() -> Self {
        Renderer::create_intern(None).expect("Cannot create renderer")
    }
}
