use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender, bounded};
use tracing::{error, info};

use crate::render::thread::RenderThread;
use crate::render::{
    CmdPrimitiveType, GpuHandle, RenderBatch, RenderCommand, RenderThreadConfig,
    RenederThreadError, ResourceId, ShaderReloadResult, SharedRenderStats,
};
use crate::window::WindowGlContext;

/// Maximum frames in flight for triple buffering
const MAX_FRAMES_IN_FLIGHT: u64 = 3;

/// Statistics from the render thread (local copy)
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
}

/// Statistics from culling/preparation
#[derive(Clone, Debug, Default)]
pub struct CullStats {
    /// Total entities submitted
    pub total_entities: u32,
    /// Entities that passed frustum culling
    pub visible_entities: u32,
    /// Entities culled
    pub culled_entities: u32,
}

pub struct Renderer {
    /// Send commands to the render thread
    command_tx: Sender<RenderCommand>,
    /// Receive fence completions from the render thread
    fence_rx: Receiver<u64>,
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
    /// Shared stats readable from main thread
    shared_stats: Arc<SharedRenderStats>,
    /// Thread-local command buffer for batching
    command_buffer: Vec<RenderCommand>,
    /// Global counter for generating unique ResourceIds
    next_resource_id: AtomicU64,
    /// Render stats
    render_stats: RenderStats,
    /// Cull stats
    cull_stats: CullStats,
    /// Active render batch
    pub(super) active_batch: Option<RenderBatch>,
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenederThreadError> {
        // Spawn the render thread with the GL context
        let config = RenderThreadConfig::default();
        // Use bounded channel for backpressure - SwapBuffers will block to sync with render thread
        let (command_tx, command_rx) = bounded(config.command_buffer_size);
        let (fence_tx, fence_rx) = bounded(config.fence_buffer_size);
        let (shader_result_tx, shader_result_rx) = bounded(16); // Buffer for shader reload results
        let (context_tx, context_rx) = bounded(1); // Only one context to return
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let shared_stats = Arc::new(SharedRenderStats::default());
        let shared_stats_clone = shared_stats.clone();

        let thread_handle = thread::Builder::new()
            .name("RenderThread".into())
            .spawn(move || {
                // Make GL context current on this thread
                let gl_context = match context.make_current() {
                    Ok(ctx) => {
                        info!("GL context made current on render thread");
                        Some(ctx)
                    }
                    Err(e) => {
                        error!("Failed to make GL context current on render thread: {}", e);
                        None
                    }
                };

                // Pass GL context to render thread for buffer swapping
                let mut render_thread = RenderThread::new(
                    command_rx,
                    fence_tx,
                    shader_result_tx,
                    context_tx,
                    running_clone,
                    shared_stats_clone,
                    gl_context,
                );
                render_thread.run();

                // GL context will be returned via channel or dropped if cleanup fails
            })
            .expect("Failed to spawn render thread");

        info!("Render thread spawned");

        info!("Render thread started successfully");

        Ok(Self {
            command_tx,
            fence_rx,
            shader_result_rx,
            context_rx,
            next_fence_id: AtomicU64::new(1),
            frames_in_flight: AtomicU64::new(0),
            running,
            thread_handle,
            shared_stats,
            command_buffer: vec![],
            next_resource_id: AtomicU64::new(1),
            render_stats: Default::default(),
            cull_stats: Default::default(),
            active_batch: None,
        })
    }

    pub fn stop(self) -> Option<WindowGlContext> {
        // We have exclusive access - shutdown and get context
        info!("Calling shutdown...");
        let returned_ctx = self.shutdown();
        info!("Render thread stopped");

        returned_ctx
    }

    /// Submit a command to the render thread
    pub fn submit(&self, cmd: RenderCommand) {
        if self.running.load(Ordering::Relaxed) {
            if let Err(e) = self.command_tx.send(cmd) {
                error!("Failed to send render command: {:?}", e);
            }
        }
    }

    /// Submit a command to the render thread without blocking.
    /// Returns true if the command was sent, false if the channel was full.
    /// Use this for commands that can be safely dropped (like resize events).
    pub fn try_submit(&self, cmd: RenderCommand) -> bool {
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

    /// Begin a new frame
    pub(in crate::render::thread) fn begin_frame_intern(&mut self) {
        self.command_buffer.clear();
    }

    /// Flush all queued commands to the render thread
    pub(in crate::render::thread) fn flush_intern(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            for cmd in self.command_buffer.drain(..) {
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

    /// Generate a new unique resource ID
    /// Uses the global counter from render_mode to avoid ID conflicts
    pub fn next_resource_id(&mut self) -> ResourceId {
        ResourceId(self.next_resource_id.fetch_add(1, Ordering::Relaxed))
    }

    /// End the current frame with triple buffering.
    /// Submits SwapBuffers and fence, blocks only if MAX_FRAMES_IN_FLIGHT are queued.
    /// Uses fence channel for proper synchronization when throttling is needed.
    pub fn end_frame_triple_buffered(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // Track ALL time spent in this function (includes channel blocking)
        let frame_end_start = std::time::Instant::now();

        // Drain completed fences (non-blocking) to update in-flight count
        while self.fence_rx.try_recv().is_ok() {
            self.frames_in_flight.fetch_sub(1, Ordering::Relaxed);
        }

        // If at limit, block waiting for one frame to complete
        while self.frames_in_flight.load(Ordering::Relaxed) >= MAX_FRAMES_IN_FLIGHT {
            match self.fence_rx.recv() {
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
        self.submit(RenderCommand::Fence { fence_id });
        self.frames_in_flight.fetch_add(1, Ordering::Relaxed);

        // Store total time spent in frame end (all blocking)
        let wait_us = frame_end_start.elapsed().as_micros() as u64;
        self.shared_stats
            .main_thread_wait_us
            .store(wait_us, Ordering::Relaxed);
    }

    /// Get current frames in flight count
    pub fn get_frames_in_flight(&self) -> u64 {
        self.frames_in_flight.load(Ordering::Relaxed)
    }

    /// Check if the render thread is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get current render stats snapshot
    pub fn get_stats(&self) -> RenderStats {
        self.shared_stats.snapshot()
    }

    /// Get total commands processed since start
    pub fn get_commands_processed(&self) -> u64 {
        self.shared_stats.commands_processed.load(Ordering::Relaxed)
    }

    /// Get total draw calls since start
    pub fn get_draw_calls(&self) -> u64 {
        self.shared_stats.draw_calls.load(Ordering::Relaxed)
    }

    /// Get total state changes since start
    pub fn get_state_changes(&self) -> u64 {
        self.shared_stats.state_changes.load(Ordering::Relaxed)
    }

    /// Get total frames rendered
    pub fn get_frame_count(&self) -> u64 {
        self.shared_stats.frame_count.load(Ordering::Relaxed)
    }

    /// Get last frame render time in microseconds
    pub fn get_last_frame_time_us(&self) -> u64 {
        self.shared_stats.last_frame_time_us.load(Ordering::Relaxed)
    }

    /// Get commands processed in last frame
    pub fn get_commands_last_frame(&self) -> u64 {
        self.shared_stats
            .commands_last_frame
            .load(Ordering::Relaxed)
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&self) -> u64 {
        self.shared_stats
            .draw_calls_last_frame
            .load(Ordering::Relaxed)
    }

    /// Get main thread wait time in microseconds (time spent waiting for render thread)
    pub fn get_main_thread_wait_us(&self) -> u64 {
        self.shared_stats
            .main_thread_wait_us
            .load(Ordering::Relaxed)
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&self) -> u64 {
        self.shared_stats
            .texture_binds_skipped
            .load(Ordering::Relaxed)
    }

    /// Reload a shader on the render thread.
    /// Returns the result with success/failure and new program handle.
    /// This blocks until the shader is compiled on the render thread.
    pub fn reload_shader(
        &self,
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

    pub fn process_batch(&mut self, mut batch: RenderBatch) {
        let mut stats = CullStats {
            total_entities: batch.entities.len() as u32,
            visible_entities: 0,
            culled_entities: 0,
        };

        // Sort entities by sort key for better batching
        batch.entities.sort_by_key(|e| e.sort_key);

        let mut current_shader: Option<u32> = None;

        for entity in &batch.entities {
            // Frustum culling
            if !batch
                .camera
                .sphere_in_frustum(entity.bounds_center, entity.bounds_radius)
            {
                stats.culled_entities += 1;
                continue;
            }

            stats.visible_entities += 1;

            // Compute MVP matrix
            let mvp = batch.camera.view_projection * entity.transform;
            let mvp_array = mvp.to_cols_array();

            // Bind shader if changed
            if current_shader != Some(entity.shader_handle) {
                self.command_buffer.push(RenderCommand::BindShader {
                    handle: GpuHandle(entity.shader_handle),
                });
                current_shader = Some(entity.shader_handle);
            }

            // Set MVP uniform
            self.command_buffer.push(RenderCommand::SetUniformMat4 {
                location: entity.mvp_location,
                value: mvp_array,
            });

            // Set model matrix uniform if needed
            if entity.model_location >= 0 {
                self.command_buffer.push(RenderCommand::SetUniformMat4 {
                    location: entity.model_location,
                    value: entity.transform.to_cols_array(),
                });
            }

            // Draw call
            self.command_buffer.push(RenderCommand::DrawMesh {
                vao: GpuHandle(entity.mesh_vao),
                index_count: entity.index_count,
                primitive: CmdPrimitiveType::Triangles,
            });
        }

        self.render_stats.batches_processed += 1;
    }

    /// Request the render thread to shutdown.
    /// Wait for the GL context to be returned from the render thread (blocking with timeout).
    /// This should be called after shutdown() to retrieve the context for
    /// restoring direct GL mode on the main thread.
    fn shutdown(self) -> Option<WindowGlContext> {
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
