use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
use crate::window::{PresentMode, WgpuStartupBundle, WindowError, WindowGlContext};

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
    /// Receive returned GL context when render thread shuts down
    context_rx: Receiver<Option<WindowGlContext>>,
    /// Ring memory the executor has uploaded, coming back for reuse.
    chunk_return_rx: Receiver<ReturnedChunk>,
    /// Next fence ID to use
    next_fence_id: AtomicU64,
    /// Number of frames currently in flight (submitted but not rendered)
    frames_in_flight: AtomicU64,
    /// Whether the render thread is running
    running: Arc<AtomicBool>,
    /// Thread handle for joining
    thread_handle: JoinHandle<()>,
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
    /// Shared with the executor: enables per-category timing when the sink is
    /// attached (dashboard mode). Kept on the renderer so `attach_stats_sink`
    /// can flip it after the executor has moved to the render thread.
    #[cfg(feature = "stats-server")]
    pub(super) category_timing: Arc<AtomicBool>,
    /// Startup backend description, filled by the render thread once up.
    backend_info: Arc<std::sync::Mutex<String>>,
    /// Generic renderer data
    pub(crate) data: RendererData,
    /// Last shader bind submitted, so identical consecutive binds can skip
    /// the channel send entirely (the executor's current_program is already
    /// that program). Mirrors the executor's program state across all bind
    /// paths: raw-handle binds and unbinds invalidate it.
    last_shader_bind: Option<u64>,
}

/// What the render thread drives: chosen once at startup.
enum RenderBackend {
    Gl(Option<WindowGlContext>),
    Wgpu(Box<WgpuStartupBundle>),
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenderThreadError> {
        Self::create_intern(RenderBackend::Gl(Some(context)))
    }

    /// Start the render thread with the wgpu backend (selected at runtime by
    /// `LTHEORY_WGPU`): the surface bundle (device/queue/surface) moves to the
    /// render thread, where `WgpuCommandExecutor` runs the same command loop
    /// as the GL path. No GL context is involved.
    pub fn start_wgpu(bundle: WgpuStartupBundle) -> Result<Self, RenderThreadError> {
        Self::create_intern(RenderBackend::Wgpu(Box::new(bundle)))
    }

    pub fn stop(self) -> Option<WindowGlContext> {
        // We have exclusive access - shutdown and get context
        info!("Calling shutdown...");
        let returned_ctx = self.shutdown();
        info!("Render thread stopped");

        returned_ctx
    }

    fn create_intern(backend: RenderBackend) -> Result<Self, RenderThreadError> {
        // Spawn the render thread with the GL context
        let config = RenderThreadConfig::default();
        // Use bounded channel for backpressure - SwapBuffers will block to sync with render thread
        let (command_tx, command_rx) = bounded(config.command_buffer_size);
        let (fence_tx, fence_rx) = bounded(config.fence_buffer_size);
        let (pacing_fence_tx, pacing_fence_rx) = bounded(config.fence_buffer_size);
        let (context_tx, context_rx) = bounded(1); // Only one context to return
        // Unbounded: the executor must never block returning ring memory.
        let (chunk_return_tx, chunk_return_rx) = unbounded();
        let (stats_tx, stats_rx) = bounded(1); // Only the latest snapshot matters
        // Unbounded: `ResourceHandle::drop` must never block or fail.
        let (destroy_tx, destroy_rx) = unbounded();
        // Bounded(1): the render thread reports whether it managed to activate
        // the GL context before doing anything else, so `create_intern` can
        // fail synchronously instead of silently running with a no-op
        // executor (see `RenderThreadError::ContextActivationFailed`).
        let (ready_tx, ready_rx) = bounded::<Result<(), WindowError>>(1);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        // Shared with the executor: flipped on by the main thread when a stats
        // sink is attached (dashboard mode), enabling per-category timing.
        let category_timing = Arc::new(AtomicBool::new(false));
        let category_timing_executor = category_timing.clone();
        let backend_info = Arc::new(std::sync::Mutex::new(String::new()));
        let backend_info_thread = backend_info.clone();

        // The wgpu bundle needs no activation handshake: with no GL context the
        // thread below reports ready immediately and builds the wgpu executor.
        let (context, wgpu_bundle) = match backend {
            RenderBackend::Gl(context) => (context, None),
            RenderBackend::Wgpu(bundle) => (None, Some(*bundle)),
        };

        let thread_handle =
            thread::Builder::new()
                .name("RenderThread".into())
                .spawn(move || {
                    // Make GL context current on this thread
                    let gl_context = if let Some(active_context) = context {
                        match active_context.make_current() {
                            Ok(ctx) => {
                                info!("GL context made current on render thread");
                                if let Err(err) = ready_tx.send(Ok(())) {
                                    error!("Cannot report GL contaxt activation success: {err}");
                                }
                                Some(ctx)
                            }
                            Err(e) => {
                                error!("Failed to make GL context current on render thread: {e}");
                                if let Err(err) = ready_tx.send(Err(e)) {
                                    error!("Cannot report GL contaxt activation failure: {err}");
                                }
                                // Nothing to run without a context - exit before
                                // constructing `RenderThread` at all.
                                return;
                            }
                        }
                    } else {
                        if let Err(err) = ready_tx.send(Ok(())) {
                            error!("Cannot report GL contaxt activation success: {err}");
                        }
                        None
                    };

                    // Pass GL context to render thread for buffer swapping
                    let render_thread = if let Some(bundle) = wgpu_bundle {
                        let info = super::render_thread::wgpu_backend_info(&bundle.adapter, &bundle.surface_config);
                        *backend_info_thread.lock().unwrap() = info;
                        RenderThread::new_wgpu(
                            command_rx,
                            fence_tx,
                            pacing_fence_tx,
                            context_tx,
                            stats_tx,
                            chunk_return_tx,
                            running_clone,
                            bundle,
                            category_timing_executor,
                        )
                    } else {
                        RenderThread::new(
                            command_rx,
                            fence_tx,
                            pacing_fence_tx,
                            context_tx,
                            stats_tx,
                            chunk_return_tx,
                            running_clone,
                            gl_context,
                            category_timing_executor,
                        )
                    };
                    let mut render_thread = render_thread.with_backend_info(backend_info_thread);
                    render_thread.run();

                    // GL context will be returned via channel or dropped if cleanup fails
                })?;

        info!("Render thread spawned");

        // Block until the render thread reports whether it managed to
        // activate the GL context - this is the one synchronous handshake
        // in an otherwise fire-and-forget startup, and it's what lets a
        // context-activation failure surface as `Err` here instead of
        // silently degrading to a no-op executor down the line.
        ready_rx.recv()??;

        info!("Render thread started successfully");

        let mut renderer = Self {
            command_tx,
            fence_rx,
            pacing_fence_rx,
            context_rx,
            chunk_return_rx,
            next_fence_id: AtomicU64::new(1),
            frames_in_flight: AtomicU64::new(0),
            running,
            thread_handle,
            stats_rx,
            last_stats: RenderStats::default(),
            main_thread_wait_us: 0,
            send_blocked_us: 0,
            send_block_count: 0,
            channel_high_water: 0,
            #[cfg(feature = "stats-server")]
            stats_sink: None,
            #[cfg(feature = "stats-server")]
            category_timing,
            backend_info,
            data: RendererData::new(destroy_tx, destroy_rx),
            last_shader_bind: None,
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
        let ids: Vec<_> = self.data.destroy_rx.try_iter().collect();

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

    /// `key=value` lines describing the backend (empty until the render
    /// thread is up).
    pub fn backend_info_intern(&self) -> String {
        let info = self.backend_info.lock().map(|s| s.clone()).unwrap_or_default();
        if info.is_empty() {
            info
        } else {
            format!("{info}renderer_mode=threaded
")
        }
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

    /// Request the render thread to shutdown.
    /// Wait for the GL context to be returned from the render thread (blocking with timeout).
    /// This should be called after shutdown() to retrieve the context for
    /// restoring direct GL mode on the main thread.
    fn shutdown(mut self) -> Option<WindowGlContext> {
        if self.running.load(Ordering::Relaxed) {
            info!("Requesting render thread shutdown");
            self.submit(RenderCommand::Shutdown);
            self.running.store(false, Ordering::Relaxed);

            // Wait for thread to finish
            if let Err(e) = &self.thread_handle.join() {
                error!("Render thread panicked: {e:?}");
            } else {
                // Wait up to 5 seconds for the context to be returned
                const CONTEXT_WAIT_TIMEOUT_SEC: u64 = 5;
                match self
                    .context_rx
                    .recv_timeout(Duration::from_secs(CONTEXT_WAIT_TIMEOUT_SEC))
                {
                    Ok(ctx) => return ctx,
                    Err(e) => {
                        error!("Timeout or error waiting for GL context return: {e:?}");
                    }
                }
            }
        }
        None
    }

    /// Get the GL context returned from the render thread after shutdown (non-blocking).
    /// This should be called after shutdown() to retrieve the context for
    /// restoring direct GL mode on the main thread.
    pub fn take_returned_context(&self) -> Option<WindowGlContext> {
        // Try to receive the context (non-blocking since shutdown already waited)
        self.context_rx.try_recv().unwrap_or_default()
    }
}

// =========================================================================
// Per-command API - one method per `RenderCommand` variant that any code
// outside `render::thread` needs. Each constructs the matching
// `RenderCommand` and calls `submit`/`try_submit`; see `renderer_immediate.rs`
// for the immediate-mode counterpart that skips the enum entirely.
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
        // The executor's current program/pipeline changed behind the old
        // bind-skip cache.
        self.last_shader_bind = None;
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

    /// Block until every previously-submitted GL command has completed
    /// (`glFinish`).
    pub fn gl_finish(&mut self) {
        self.submit(RenderCommand::Flush);
    }

    /// A `Renderer` with no GL context at all - every command becomes a
    /// no-op (see `CommandExecutor::has_gl_context`). Only for unit tests
    /// that exercise CPU-side logic (e.g. HmGui layout) and have no window
    /// to draw a real `WindowGlContext` from.
    #[cfg(test)]
    pub fn new_headless() -> Self {
        Renderer::create_intern(RenderBackend::Gl(None)).expect("Cannot create renderer")
    }
}
