use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender, bounded, unbounded};
use tracing::{error, info};

#[cfg(feature = "stats-server")]
use crate::render::StatsSink;
use crate::render::thread::{RenderThread, process_batch_intern};
use crate::render::{
    BlendMode, ClipManager, CmdPrimitiveType, CullFace, DrawState, GpuHandle, ImmVertex,
    InstanceData, PrimitiveBuilder, RenderCommand, RenderStateIntern, RenderStats,
    RenderTargetStack, RenderThreadConfig, RenderThreadError, RendererData, ResourceHandle,
    ResourceId, ShaderErrorQueue, ShaderReloadResult, ShaderVarMap, TexFilter, TexFormat,
    TexWrapMode, VertexFormat, VpStack,
};
use crate::window::{WindowError, WindowGlContext};

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
    /// Receive shader reload results from the render thread
    shader_result_rx: Receiver<ShaderReloadResult>,
    /// Receive returned GL context when render thread shuts down
    context_rx: Receiver<Option<WindowGlContext>>,
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
    pub(super) send_blocked_us_last_frame: u64,
    /// Number of `submit()` calls that blocked on a full channel this frame
    pub(super) send_block_count_last_frame: u64,
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
    /// Generic renderer data
    pub(crate) data: RendererData,
    /// Last shader bind submitted, so identical consecutive binds can skip
    /// the channel send entirely (the executor's current_program is already
    /// that program). Mirrors the executor's program state across all bind
    /// paths: raw-handle binds and unbinds invalidate it.
    last_shader_bind: Option<u64>,
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenderThreadError> {
        Self::create_intern(Some(context))
    }

    /// A `Renderer` with no GL context at all - every command becomes a
    /// no-op (see `CommandExecutor::has_gl_context`). Only for unit tests
    /// that exercise CPU-side logic (e.g. HmGui layout) and have no window
    /// to draw a real `WindowGlContext` from.
    #[cfg(test)]
    pub fn new_headless() -> Self {
        Self::create_intern(None).expect("Cannot create renderer")
    }

    pub fn stop(self) -> Option<WindowGlContext> {
        // We have exclusive access - shutdown and get context
        info!("Calling shutdown...");
        let returned_ctx = self.shutdown();
        info!("Render thread stopped");

        returned_ctx
    }

    fn create_intern(context: Option<WindowGlContext>) -> Result<Self, RenderThreadError> {
        // Spawn the render thread with the GL context
        let config = RenderThreadConfig::default();
        // Use bounded channel for backpressure - SwapBuffers will block to sync with render thread
        let (command_tx, command_rx) = bounded(config.command_buffer_size);
        let (fence_tx, fence_rx) = bounded(config.fence_buffer_size);
        let (pacing_fence_tx, pacing_fence_rx) = bounded(config.fence_buffer_size);
        let (shader_result_tx, shader_result_rx) = bounded(16); // Buffer for shader reload results
        let (context_tx, context_rx) = bounded(1); // Only one context to return
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

        let thread_handle = thread::Builder::new()
            .name("RenderThread".into())
            .spawn(move || {
                // Make GL context current on this thread
                let gl_context = if let Some(active_context) = context {
                    match active_context.make_current() {
                        Ok(ctx) => {
                            info!("GL context made current on render thread");
                            let _ = ready_tx.send(Ok(()));
                            Some(ctx)
                        }
                        Err(e) => {
                            error!("Failed to make GL context current on render thread: {e}");
                            let _ = ready_tx.send(Err(e));
                            // Nothing to run without a context - exit before
                            // constructing `RenderThread` at all.
                            return;
                        }
                    }
                } else {
                    let _ = ready_tx.send(Ok(()));
                    None
                };

                // Pass GL context to render thread for buffer swapping
                let mut render_thread = RenderThread::new(
                    command_rx,
                    fence_tx,
                    pacing_fence_tx,
                    shader_result_tx,
                    context_tx,
                    stats_tx,
                    running_clone,
                    gl_context,
                    category_timing_executor,
                );
                render_thread.run();

                // GL context will be returned via channel or dropped if cleanup fails
            })
            .expect("Failed to spawn render thread");

        info!("Render thread spawned");

        // Block until the render thread reports whether it managed to
        // activate the GL context - this is the one synchronous handshake
        // in an otherwise fire-and-forget startup, and it's what lets a
        // context-activation failure surface as `Err` here instead of
        // silently degrading to a no-op executor down the line.
        ready_rx
            .recv()
            .expect("Render thread did not report context-activation status")?;

        info!("Render thread started successfully");

        Ok(Self {
            command_tx,
            fence_rx,
            pacing_fence_rx,
            shader_result_rx,
            context_rx,
            next_fence_id: AtomicU64::new(1),
            frames_in_flight: AtomicU64::new(0),
            running,
            thread_handle,
            stats_rx,
            last_stats: RenderStats::default(),
            main_thread_wait_us: 0,
            send_blocked_us_last_frame: 0,
            send_block_count_last_frame: 0,
            channel_high_water: 0,
            #[cfg(feature = "stats-server")]
            stats_sink: None,
            #[cfg(feature = "stats-server")]
            category_timing,
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
                shader_errors: ShaderErrorQueue::new(),
                shader_watcher: None,
                ao_shader: None,
                occlusion_shader: None,
                irmap_shader: None,
            },
            last_shader_bind: None,
        })
    }

    /// Submit a command to the render thread
    fn submit(&mut self, cmd: RenderCommand) {
        if self.running.load(Ordering::Relaxed) {
            // Fast path: non-blocking try_send. Only when the bounded channel
            // is full do we fall back to a blocking send — and only then do we
            // time the stall, so `send_blocked_us_last_frame` measures real
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
                Err(crossbeam::channel::TrySendError::Full(cmd)) => {
                    let depth = self.command_tx.len() as u64;
                    if depth > self.channel_high_water {
                        self.channel_high_water = depth;
                    }
                    let start = std::time::Instant::now();
                    if let Err(e) = self.command_tx.send(cmd) {
                        error!("Failed to send render command: {:?}", e);
                    }
                    let blocked_us = start.elapsed().as_micros() as u64;
                    self.send_blocked_us_last_frame += blocked_us;
                    self.send_block_count_last_frame += 1;
                }
                Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
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
                Err(crossbeam::channel::TrySendError::Full(_)) => {
                    // Channel full, command dropped (will be retried next frame)
                    false
                }
                Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                    error!("Render thread disconnected");
                    false
                }
            }
        } else {
            false
        }
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

        self.submit(RenderCommand::DestroyResources { ids });
    }

    /// End the current frame with triple buffering.
    /// Submits SwapBuffers and fence, blocks only if MAX_FRAMES_IN_FLIGHT are queued.
    /// Uses fence channel for proper synchronization when throttling is needed.
    pub fn end_frame_triple_buffered(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

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

        // Store total time spent in frame end (all blocking)
        self.main_thread_wait_us = frame_end_start.elapsed().as_micros() as u64;

        // Publish the combined snapshot to the dashboard sink (if attached)
        #[cfg(feature = "stats-server")]
        self.publish_stats_snapshot();

        // Reset per-frame producer counters for the next frame
        self.send_blocked_us_last_frame = 0;
        self.send_block_count_last_frame = 0;
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
        self.last_stats.draw_calls
    }

    /// Get total state changes since start
    pub fn get_state_changes(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.state_changes
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
        self.last_stats.commands_last_frame
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.draw_calls_last_frame
    }

    /// Get main thread wait time in microseconds (time spent waiting for render thread)
    pub fn get_main_thread_wait_us(&self) -> u64 {
        self.main_thread_wait_us
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&mut self) -> u64 {
        self.refresh_stats();
        self.last_stats.texture_binds_skipped
    }

    /// Reload a shader on the render thread.
    /// Returns the result with success/failure and new program handle.
    /// This blocks until the shader is compiled on the render thread.
    pub fn reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> ShaderReloadResult {
        if !self.running.load(Ordering::Relaxed) {
            return ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Render thread not running".to_string()),
                program: 0,
            };
        }

        // Send the reload command
        self.submit(RenderCommand::ReloadShader {
            shader_key: shader_key.to_string(),
            vertex_src: vertex_src.to_string(),
            fragment_src: fragment_src.to_string(),
        });

        // Wait for the result (blocking)
        match self.shader_result_rx.recv() {
            Ok(result) => result,
            Err(_) => ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Channel closed while waiting for shader result".to_string()),
                program: 0,
            },
        }
    }

    pub fn process_batch(&mut self) {
        process_batch_intern(&mut self.data.active_batch, &mut self.data.command_buffer);
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
                error!("Render thread panicked: {:?}", e);
            } else {
                // Wait up to 5 seconds for the context to be returned
                match self.context_rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(ctx) => return ctx,
                    Err(e) => {
                        error!("Timeout or error waiting for GL context return: {:?}", e);
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
    /// Begin a new frame
    pub(in crate::render::thread) fn begin_frame_intern(&mut self) {
        self.data.command_buffer.clear();
    }

    /// Flush all queued commands to the render thread
    pub(in crate::render::thread) fn flush_intern(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            // TODO: send vector of commands instead of one by one
            for cmd in self.data.command_buffer.drain(..) {
                if let Err(e) = self.command_tx.send(cmd) {
                    error!("Failed to send render command: {:?}", e);
                    break;
                }
            }
        }
    }

    /// Synchronize with the render thread (wait for all commands to complete)
    pub(in crate::render::thread) fn sync_intern(&mut self) -> bool {
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

    pub fn set_viewport_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
        });
    }

    pub fn set_scissor_intern(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetScissor {
            x,
            y,
            width,
            height,
        });
    }

    pub fn enable_scissor_intern(&mut self, enable: bool) {
        self.submit(RenderCommand::EnableScissor(enable));
    }

    pub fn set_blend_mode_intern(&mut self, mode: BlendMode) {
        self.submit(RenderCommand::SetBlendMode(mode));
    }

    pub fn set_cull_face_intern(&mut self, face: CullFace) {
        self.submit(RenderCommand::SetCullFace(face));
    }

    pub fn set_depth_test_intern(&mut self, enable: bool) {
        self.submit(RenderCommand::SetDepthTest(enable));
    }

    pub fn set_depth_writable_intern(&mut self, enable: bool) {
        self.submit(RenderCommand::SetDepthWritable(enable));
    }

    pub fn set_wireframe_intern(&mut self, enable: bool) {
        self.submit(RenderCommand::SetWireframe(enable));
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.submit(RenderCommand::SetLineWidth(width));
    }

    pub fn set_point_size(&mut self, size: f32) {
        self.submit(RenderCommand::SetPointSize(size));
    }

    // === Shader Operations ===

    pub fn bind_shader_intern(&mut self, handle: GpuHandle) {
        self.submit(RenderCommand::BindShader { handle });
        self.last_shader_bind = Some(handle.0 as u64);
    }

    pub fn bind_shader_by_resource(&mut self, id: ResourceId, shader_key: Option<String>) {
        // Skip identical consecutive binds: the executor's current_program is
        // already this program (uniform/texture commands between two binds of
        // the same shader don't change the program), so the command would
        // only be deduped on the render thread anyway. Saves the channel
        // send + executor dispatch per redundant bind (~1,900/frame in the
        // main menu, where every mesh re-binds its material's shader).
        if shader_key.is_none() && self.last_shader_bind == Some(id.0) {
            return;
        }
        self.submit(RenderCommand::BindShaderByResource { id, shader_key });
        self.last_shader_bind = Some(id.0);
    }

    pub fn unbind_shader_intern(&mut self) {
        self.submit(RenderCommand::UnbindShader);
        self.last_shader_bind = None;
    }

    pub fn set_uniform_int_intern(&mut self, location: i32, value: i32) {
        self.submit(RenderCommand::SetUniformInt { location, value });
    }

    pub fn set_uniform_int2(&mut self, location: i32, value: [i32; 2]) {
        self.submit(RenderCommand::SetUniformInt2 { location, value });
    }

    pub fn set_uniform_int3(&mut self, location: i32, value: [i32; 3]) {
        self.submit(RenderCommand::SetUniformInt3 { location, value });
    }

    pub fn set_uniform_int4(&mut self, location: i32, value: [i32; 4]) {
        self.submit(RenderCommand::SetUniformInt4 { location, value });
    }

    pub fn set_uniform_float_intern(&mut self, location: i32, value: f32) {
        self.submit(RenderCommand::SetUniformFloat { location, value });
    }

    pub fn set_uniform_float2_intern(&mut self, location: i32, value: [f32; 2]) {
        self.submit(RenderCommand::SetUniformFloat2 { location, value });
    }

    pub fn set_uniform_float3_intern(&mut self, location: i32, value: [f32; 3]) {
        self.submit(RenderCommand::SetUniformFloat3 { location, value });
    }

    pub fn set_uniform_float4_intern(&mut self, location: i32, value: [f32; 4]) {
        self.submit(RenderCommand::SetUniformFloat4 { location, value });
    }

    pub fn set_uniform_mat4(&mut self, location: i32, value: [f32; 16]) {
        self.submit(RenderCommand::SetUniformMat4 { location, value });
    }

    pub fn set_instance_uniforms(
        &mut self,
        world_loc: i32,
        world_it_loc: i32,
        scale_loc: i32,
        world: [f32; 16],
        world_it: [f32; 16],
        scale: f32,
    ) {
        self.submit(RenderCommand::SetInstanceUniforms {
            world_loc,
            world_it_loc,
            scale_loc,
            world,
            world_it,
            scale,
        });
    }

    // === Texture Operations ===

    pub fn bind_texture_2d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.submit(RenderCommand::BindTexture2D { slot, handle });
    }

    pub fn bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.submit(RenderCommand::BindTexture2DByResource { slot, id });
    }

    pub fn bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.submit(RenderCommand::BindTexture1DByResource { slot, id });
    }

    pub fn bind_texture_3d_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.submit(RenderCommand::BindTexture3D { slot, handle });
    }

    pub fn bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.submit(RenderCommand::BindTexture3DByResource { slot, id });
    }

    pub fn bind_texture_cube_intern(&mut self, slot: u32, handle: GpuHandle) {
        self.submit(RenderCommand::BindTextureCube { slot, handle });
    }

    pub fn bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.submit(RenderCommand::BindTextureCubeByResource { slot, id });
    }

    pub fn unbind_texture_intern(&mut self, slot: u32) {
        self.submit(RenderCommand::UnbindTexture { slot });
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
        self.submit(RenderCommand::UpdateTexture2DDataByResource {
            id,
            width,
            height,
            internal_format,
            pixel_format,
            data_format,
            data,
        });
    }

    pub fn set_texture_2d_anisotropy_by_resource(&mut self, id: ResourceId, factor: f32) {
        self.submit(RenderCommand::SetTexture2DAnisotropyByResource { id, factor });
    }

    pub fn set_texture_2d_mip_range_by_resource(
        &mut self,
        id: ResourceId,
        min_level: i32,
        max_level: i32,
    ) {
        self.submit(RenderCommand::SetTexture2DMipRangeByResource {
            id,
            min_level,
            max_level,
        });
    }

    pub fn set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        self.submit(RenderCommand::SetTexel1DByResource { id, x, color });
    }

    pub fn set_texel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32, color: [f32; 4]) {
        self.submit(RenderCommand::SetTexel2DByResource { id, x, y, color });
    }

    pub fn set_texture_mag_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.submit(RenderCommand::SetTextureMagFilterByResource { id, filter });
    }

    pub fn set_texture_min_filter_by_resource(&mut self, id: ResourceId, filter: TexFilter) {
        self.submit(RenderCommand::SetTextureMinFilterByResource { id, filter });
    }

    pub fn set_texture_wrap_mode_by_resource(&mut self, id: ResourceId, mode: TexWrapMode) {
        self.submit(RenderCommand::SetTextureWrapModeByResource { id, mode });
    }

    pub fn generate_mipmap_by_resource(&mut self, id: ResourceId) {
        self.submit(RenderCommand::GenerateMipmapByResource { id });
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
        self.submit(RenderCommand::UpdateTexture1DDataByResource {
            id,
            width,
            internal_format,
            pixel_format,
            data_format,
            data,
        });
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
        self.submit(RenderCommand::UpdateTexture3DDataByResource {
            id,
            width,
            height,
            depth,
            internal_format,
            pixel_format,
            data_format,
            data,
        });
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
        self.submit(RenderCommand::UpdateTextureCubeFaceDataByResource {
            id,
            face,
            level,
            size,
            internal_format,
            pixel_format,
            data_format,
            data,
        });
    }

    pub fn copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        internal_format: i32,
        width: i32,
        height: i32,
    ) {
        self.submit(RenderCommand::CopyTexture2DFromFramebufferByResource {
            id,
            internal_format,
            width,
            height,
        });
    }

    pub fn read_texture_1d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::ReadTexture1DData {
            id,
            pixel_format,
            data_format,
            reply_tx: tx,
        });
        rx.recv().unwrap_or_default()
    }

    pub fn read_texture_2d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::ReadTexture2DData {
            id,
            pixel_format,
            data_format,
            reply_tx: tx,
        });
        rx.recv().unwrap_or_default()
    }

    pub fn read_texture_3d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::ReadTexture3DData {
            id,
            pixel_format,
            data_format,
            reply_tx: tx,
        });
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
        self.submit(RenderCommand::ReadTextureCubeFaceData {
            id,
            face,
            level,
            pixel_format,
            data_format,
            reply_tx: tx,
        });
        rx.recv().unwrap_or_default()
    }

    pub fn sample_pixel_2d_by_resource(&mut self, id: ResourceId, x: i32, y: i32) -> [u8; 4] {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::SamplePixel2DByResource {
            id,
            x,
            y,
            reply_tx: tx,
        });
        rx.recv().unwrap_or([0; 4])
    }

    pub fn read_framebuffer_pixels(&mut self, x: i32, y: i32, width: i32, height: i32) -> Vec<u8> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::ReadFramebufferPixels {
            x,
            y,
            width,
            height,
            reply_tx: tx,
        });
        rx.recv().unwrap_or_default()
    }

    // === Framebuffer Operations ===

    pub fn push_framebuffer(&mut self, id: u64, width: i32, height: i32) {
        self.submit(RenderCommand::PushFramebuffer { id, width, height });
    }

    pub fn pop_framebuffer(&mut self) {
        self.submit(RenderCommand::PopFramebuffer);
    }

    pub fn framebuffer_attach_texture_2d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        level: i32,
    ) {
        self.submit(RenderCommand::FramebufferAttachTexture2DByResource {
            attachment,
            id,
            level,
        });
    }

    pub fn framebuffer_attach_texture_3d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        layer: i32,
        level: i32,
    ) {
        self.submit(RenderCommand::FramebufferAttachTexture3DByResource {
            attachment,
            id,
            layer,
            level,
        });
    }

    pub fn framebuffer_attach_texture_cube_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        face: u32,
        level: i32,
    ) {
        self.submit(RenderCommand::FramebufferAttachTextureCubeByResource {
            attachment,
            id,
            face,
            level,
        });
    }

    pub fn bind_framebuffer_intern(&mut self, handle: GpuHandle) {
        self.submit(RenderCommand::BindFramebuffer { handle });
    }

    pub fn bind_default_framebuffer_intern(&mut self) {
        self.submit(RenderCommand::BindDefaultFramebuffer);
    }

    pub fn clear_intern(&mut self, color: Option<[f32; 4]>, depth: Option<f32>) {
        self.submit(RenderCommand::Clear { color, depth });
    }

    // === Drawing Operations ===

    pub fn draw_mesh_intern(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.submit(RenderCommand::DrawMesh {
            vao,
            index_count,
            primitive,
        });
    }

    /// Texture-fetch instancing: submit per-instance u32 indices into a
    /// static data texture. The data is copied into the command so the
    /// render thread owns it (Lua array reusable after the call).
    pub fn draw_instanced_indices_intern(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        indices: &[u32],
        primitive: CmdPrimitiveType,
    ) {
        self.submit(RenderCommand::DrawInstancedIndices {
            mesh_id,
            index_count,
            indices: indices.to_vec(),
            primitive,
        });
    }

    pub fn draw_mesh_instanced_intern(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.submit(RenderCommand::DrawMeshInstanced {
            vao,
            index_count,
            instance_count,
            primitive,
        });
    }

    pub fn draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.submit(RenderCommand::DrawMeshByResource {
            id,
            index_count,
            primitive,
        });
    }

    pub fn draw_instanced_with_data_intern(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        instances: Vec<InstanceData>,
        primitive: CmdPrimitiveType,
    ) {
        self.submit(RenderCommand::DrawInstancedWithData {
            mesh_id,
            index_count,
            instances,
            primitive,
        });
    }

    pub fn draw_immediate(&mut self, primitive: CmdPrimitiveType, vertices: Vec<ImmVertex>) {
        self.submit(RenderCommand::DrawImmediate {
            primitive,
            vertices,
        });
    }

    // === Resource Creation ===

    pub fn create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
    ) -> Option<String> {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::CreateShader {
            id,
            vertex_src,
            fragment_src,
            reply_tx: tx,
        });
        rx.recv()
            .unwrap_or_else(|_| Some("Renderer channel closed".to_string()))
    }

    pub fn get_uniform_location_by_resource(&mut self, id: ResourceId, name: Arc<str>) -> i32 {
        let (tx, rx) = bounded(1);
        self.submit(RenderCommand::GetUniformLocationByResource {
            id,
            name,
            reply_tx: tx,
        });
        rx.recv().unwrap_or(-1)
    }

    pub fn create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.submit(RenderCommand::CreateTexture1D {
            id,
            width,
            format,
            data,
        });
    }

    pub fn create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        self.submit(RenderCommand::CreateTexture2D {
            id,
            width,
            height,
            format,
            data,
        });
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
        self.submit(RenderCommand::CreateTexture3D {
            id,
            width,
            height,
            depth,
            format,
            data,
        });
    }

    pub fn create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        self.submit(RenderCommand::CreateTextureCube { id, size, format });
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

    // === Uniform Buffer Objects ===

    pub fn create_camera_ubo_intern(&mut self) {
        self.submit(RenderCommand::CreateCameraUBO);
    }

    pub fn update_camera_ubo_intern(&mut self, data: Box<[u8; 288]>) {
        self.submit(RenderCommand::UpdateCameraUBO { data });
    }

    pub fn create_material_ubo_intern(&mut self) {
        self.submit(RenderCommand::CreateMaterialUBO);
    }

    pub fn update_material_ubo_intern(&mut self, data: [u8; 32]) {
        self.submit(RenderCommand::UpdateMaterialUBO { data });
    }

    pub fn create_light_ubo_intern(&mut self) {
        self.submit(RenderCommand::CreateLightUBO);
    }

    pub fn update_light_ubo_intern(&mut self, data: [u8; 32]) {
        self.submit(RenderCommand::UpdateLightUBO { data });
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

    /// Block until every previously-submitted GL command has completed
    /// (`glFinish`). Named to avoid colliding with `flush()`/`flush_intern`,
    /// which drains the CPU-side batch command buffer - an unrelated concept.
    pub fn gl_finish(&mut self) {
        self.submit(RenderCommand::Flush);
    }
}
