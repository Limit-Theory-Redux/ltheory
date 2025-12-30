//! # Render Context
//!
//! Manages the multithreaded rendering subsystem, encapsulating the render thread,
//! worker pool, and associated state. This provides a clean separation of concerns
//! between the Engine (which manages windows, input, Lua) and the rendering subsystem.
//!
//! ## Design
//!
//! The `RenderContext` owns the render thread handle and worker pool. It does NOT
//! own the GL context directly - that belongs to the window system. Instead:
//!
//! - `start()` receives `RenderThreadGlData` extracted from the window
//! - `stop()` returns `ReturnedGlContext` for the caller to restore to the window
//!
//! This avoids circular dependencies and keeps the GL context ownership clear.

use std::sync::Arc;
use tracing::*;

use super::{
    RenderThreadHandle, RenderThreadConfig, spawn_render_thread, RenderCommand,
    ReturnedGlContext, enable_command_mode, disable_command_mode, set_render_handle,
    clear_render_handle, WorkerPoolHandle, WorkerPoolConfig, spawn_worker_pool,
};
use crate::window::RenderThreadGlData;

/// Manages the multithreaded rendering subsystem.
///
/// This struct encapsulates all state related to the render thread and worker pool,
/// providing a clean API for starting, stopping, and querying the render subsystem.
pub struct RenderContext {
    /// Handle to the dedicated render thread (when active)
    thread_handle: Option<Arc<RenderThreadHandle>>,
    /// Handle to the worker pool for parallel render preparation
    worker_pool: Option<WorkerPoolHandle>,
    /// Whether multithreaded rendering is currently active
    active: bool,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderContext {
    /// Create a new inactive render context.
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            worker_pool: None,
            active: false,
        }
    }

    /// Start the render thread with the provided GL context.
    ///
    /// This transfers ownership of the GL context to the render thread.
    /// The context will be returned when `stop()` is called.
    ///
    /// # Arguments
    /// * `gl_data` - GL context data extracted from the window system
    ///
    /// # Returns
    /// `true` if successfully started, `false` if already running
    pub fn start(&mut self, gl_data: RenderThreadGlData) -> bool {
        if self.thread_handle.is_some() {
            warn!("Render thread already started");
            return false;
        }

        // Spawn the render thread with the GL context
        let config = RenderThreadConfig::default();
        let handle = Arc::new(spawn_render_thread(config, Some(gl_data)));

        // Set the global render handle for command submission
        set_render_handle(handle.clone());

        // Store our Arc reference
        self.thread_handle = Some(handle);
        self.active = true;

        // Spawn worker pool for parallel render preparation
        let worker_config = WorkerPoolConfig::default();
        let worker_pool = spawn_worker_pool(worker_config);
        info!("Worker pool spawned with {} workers", worker_pool.num_workers());
        self.worker_pool = Some(worker_pool);

        // Enable command mode globally
        enable_command_mode();

        info!("Render thread started successfully");
        true
    }

    /// Stop the render thread and return the GL context.
    ///
    /// This shuts down the render thread and worker pool, returning the GL context
    /// so it can be restored to the window system.
    ///
    /// # Returns
    /// The GL context data if successfully stopped, `None` if not running or failed
    pub fn stop(&mut self) -> Option<ReturnedGlContext> {
        // Shutdown worker pool first
        if let Some(worker_pool) = self.worker_pool.take() {
            info!("Shutting down worker pool...");
            worker_pool.shutdown();
        }

        let handle = self.thread_handle.take()?;

        // Disable command mode first so no new commands are submitted
        disable_command_mode();

        // Clear the global handle first to drop its Arc reference
        clear_render_handle();

        // Now try to get exclusive access for proper shutdown
        info!("Attempting to get exclusive access to render thread handle...");
        let returned_ctx = match Arc::try_unwrap(handle) {
            Ok(mut h) => {
                // We have exclusive access - shutdown and get context
                info!("Got exclusive access, calling shutdown...");
                h.shutdown();
                info!("Shutdown complete, waiting for context...");

                // Wait for the context to be returned
                h.wait_for_returned_context()
            }
            Err(arc) => {
                // Other Arc references still exist - this shouldn't happen normally
                warn!(
                    "Other Arc references exist ({} strong refs), forcing shutdown anyway",
                    Arc::strong_count(&arc)
                );

                // Send shutdown command to stop the render thread
                arc.submit(RenderCommand::Shutdown);

                // Use blocking wait for the context to be returned
                info!("Waiting for context from render thread...");
                let ctx = arc.wait_for_returned_context();

                // Drop the arc (will call shutdown in Drop impl if needed)
                drop(arc);
                ctx
            }
        };

        self.active = false;
        info!("Render thread stopped");

        returned_ctx
    }

    /// Check if the render thread is currently active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active && self.thread_handle.is_some()
    }

    /// Get a reference to the render thread handle, if active.
    #[inline]
    pub fn handle(&self) -> Option<&RenderThreadHandle> {
        self.thread_handle.as_ref().map(|arc| arc.as_ref())
    }

    /// Get a reference to the worker pool, if active.
    #[inline]
    pub fn worker_pool(&self) -> Option<&WorkerPoolHandle> {
        self.worker_pool.as_ref()
    }

    // ---- Stats accessors ----

    /// Get total commands processed by the render thread.
    #[inline]
    pub fn get_commands_processed(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_commands_processed())
    }

    /// Get total draw calls executed by the render thread.
    #[inline]
    pub fn get_draw_calls(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_draw_calls())
    }

    /// Get total state changes on the render thread.
    #[inline]
    pub fn get_state_changes(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_state_changes())
    }

    /// Get total frames rendered by the render thread.
    #[inline]
    pub fn get_frame_count(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_frame_count())
    }

    /// Get the last frame render time in microseconds.
    #[inline]
    pub fn get_last_frame_time_us(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_last_frame_time_us())
    }

    /// Get the last frame render time in milliseconds.
    #[inline]
    pub fn get_last_frame_time_ms(&self) -> f64 {
        self.get_last_frame_time_us() as f64 / 1000.0
    }

    /// Get commands processed in the last frame.
    #[inline]
    pub fn get_commands_last_frame(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_commands_last_frame())
    }

    /// Get draw calls executed in the last frame.
    #[inline]
    pub fn get_draw_calls_last_frame(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_draw_calls_last_frame())
    }

    /// Get total texture binds skipped due to caching.
    #[inline]
    pub fn get_texture_binds_skipped(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_texture_binds_skipped())
    }

    /// Get main thread wait time in microseconds.
    #[inline]
    pub fn get_main_thread_wait_us(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_main_thread_wait_us())
    }

    /// Get main thread wait time in milliseconds.
    #[inline]
    pub fn get_main_thread_wait_ms(&self) -> f64 {
        self.get_main_thread_wait_us() as f64 / 1000.0
    }

    /// Get current frames in flight (submitted but not yet rendered).
    #[inline]
    pub fn get_frames_in_flight(&self) -> u64 {
        self.thread_handle
            .as_ref()
            .map_or(0, |h| h.get_frames_in_flight())
    }
}
