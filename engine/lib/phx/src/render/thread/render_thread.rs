use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::render::thread::command_executor_wgpu::WgpuCommandExecutor;
use crate::render::thread::{CommandExecutor, CommandReply};
use crate::render::{RenderCommand, RenderStats, ReturnedChunk};
use crate::window::{WgpuStartupBundle, WindowActiveGlContext, WindowGlContext};

/// Drives a [`CommandExecutor`] on a dedicated thread.
///
/// This type owns only the plumbing: it pulls commands off a channel, hands
/// them to the executor, and forwards whatever the executor answers back to
/// the main thread. All GL work and all GPU state live in the executor, which
/// is why the same executor can also be driven inline in immediate mode.
pub struct RenderThread {
    command_rx: Receiver<RenderCommand>,
    fence_tx: Sender<u64>,
    /// Reply channel for `RenderCommand::PacingFence`, kept separate from
    /// `fence_tx` so frame-pacing fences and `sync_intern`'s blocking
    /// round-trip fences can never be consumed by the wrong consumer (see
    /// `RenderCommand::PacingFence`'s docs).
    pacing_fence_tx: Sender<u64>,
    /// Channel to return GL context to main thread on shutdown
    context_tx: Sender<Option<WindowGlContext>>,
    /// Channel to publish a stats snapshot to the main thread on every frame
    stats_tx: Sender<RenderStats>,
    /// Channel returning uploaded ring memory to the main thread
    chunk_return_tx: Sender<ReturnedChunk>,
    running: Arc<AtomicBool>,
    /// Executes commands in thread
    executor: CommandExecutor,
    /// Parallel wgpu backend; when set, commands go to it instead of the GL
    /// executor (which then has no context). Exactly one of the two is active.
    wgpu_executor: Option<WgpuCommandExecutor>,
}

impl RenderThread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        pacing_fence_tx: Sender<u64>,
        context_tx: Sender<Option<WindowGlContext>>,
        stats_tx: Sender<RenderStats>,
        chunk_return_tx: Sender<ReturnedChunk>,
        running: Arc<AtomicBool>,
        gl_context: Option<WindowActiveGlContext>,
        category_timing: Arc<AtomicBool>,
    ) -> Self {
        Self {
            command_rx,
            fence_tx,
            pacing_fence_tx,
            context_tx,
            stats_tx,
            chunk_return_tx,
            running,
            executor: CommandExecutor::new_with_timing(gl_context, category_timing),
            wgpu_executor: None,
        }
    }

    /// wgpu flavor: the executor is built from the surface bundle (device,
    /// queue, surface and initial size all move here; no GL anywhere).
    #[allow(clippy::too_many_arguments)]
    pub fn new_wgpu(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        pacing_fence_tx: Sender<u64>,
        context_tx: Sender<Option<WindowGlContext>>,
        stats_tx: Sender<RenderStats>,
        chunk_return_tx: Sender<ReturnedChunk>,
        running: Arc<AtomicBool>,
        bundle: WgpuStartupBundle,
        category_timing: Arc<AtomicBool>,
    ) -> Self {
        let (width, height) = (bundle.surface_config.width, bundle.surface_config.height);
        let executor = WgpuCommandExecutor::with_device(Some(bundle.device), Some(bundle.queue))
            .with_surface(bundle.surface, bundle.surface_config, width, height);
        executor.set_category_timing(category_timing.clone());
        Self {
            command_rx,
            fence_tx,
            pacing_fence_tx,
            context_tx,
            stats_tx,
            chunk_return_tx,
            running,
            executor: CommandExecutor::new_with_timing(None, category_timing),
            wgpu_executor: Some(executor),
        }
    }

    /// Main render loop
    pub fn run(&mut self) {
        info!("Render thread started");

        // Only initialize GL resources if we have a valid context
        if self.wgpu_executor.is_some() {
            info!("Render thread running the wgpu backend");
        } else if self.executor.has_gl_context() {
            self.executor.init_gl();
        } else {
            warn!("Render thread running without GL context - commands will be no-ops");
        }

        while self.running.load(Ordering::Relaxed) {
            // Measure render-thread idle time: how long we block waiting for
            // commands (the producer-starvation gap). A fast-path recv on a
            // populated channel returns in a few µs; a genuinely blocked recv
            // waits hundreds of µs+. Only waits above the threshold count as
            // starvation, so fast-path scheduling noise stays out.
            const STARVATION_THRESHOLD_US: u64 = 20;

            let recv_start = std::time::Instant::now();
            let cmd = self.command_rx.recv();
            let recv_wait_us = recv_start.elapsed().as_micros() as u64;

            match cmd {
                Ok(cmd) => {
                    if matches!(cmd, RenderCommand::Shutdown) {
                        info!("Render thread received shutdown command");
                        break;
                    }

                    // Per-frame command count (record_command does this when
                    // the stats-server feature is on).
                    #[cfg(not(feature = "stats-server"))]
                    if self.wgpu_executor.is_none() {
                        self.executor.this_frame_stats.commands += 1;
                    }

                    if self.wgpu_executor.is_none() && recv_wait_us >= STARVATION_THRESHOLD_US {
                        self.executor.this_frame_stats.recv_wait_us += recv_wait_us;
                        self.executor.this_frame_stats.recv_wait_count += 1;
                    }

                    let reply = if let Some(wgpu) = self.wgpu_executor.as_mut() {
                        wgpu.execute(cmd)
                    } else {
                        self.executor.execute(cmd)
                    };
                    self.dispatch(reply);
                    // Ring memory the command finished uploading goes back to
                    // the main thread for reuse.
                    if self.wgpu_executor.is_none() {
                        for chunk in self.executor.take_returned_chunks() {
                            let _ = self.chunk_return_tx.send(chunk);
                        }
                    }
                }
                Err(_) => {
                    debug!("Command channel closed, render thread exiting");
                    break;
                }
            }
        }

        let context = if let Some(wgpu) = self.wgpu_executor.as_ref() {
            // wgpu: nothing to hand back (no GL context exists).
            info!("Render thread stopped. wgpu stats: {:?}", wgpu.stats());
            None
        } else {
            let context = self.executor.cleanup();
            info!("Render thread stopped. Stats: {:?}", self.executor.stats());
            context
        };
        if let Err(e) = self.context_tx.send(context) {
            error!("Failed to signal main thread on shutdown: {e:?}");
        }
    }

    /// Forward an executor answer over the channel it belongs to.
    fn dispatch(&self, reply: CommandReply) {
        match reply {
            CommandReply::None => {}
            CommandReply::Fence(fence_id) => {
                if let Err(e) = self.fence_tx.send(fence_id) {
                    warn!("Failed to send fence signal: {e:?}");
                }
            }
            CommandReply::PacingFence(fence_id) => {
                if let Err(e) = self.pacing_fence_tx.send(fence_id) {
                    warn!("Failed to send pacing fence signal: {e:?}");
                }
            }
            CommandReply::Stats(stats) => {
                // Best-effort: if the main thread hasn't drained the last
                // snapshot yet, dropping this one is fine, the next frame's
                // will supersede it. Only warn if the channel is gone.
                if let Err(e) = self.stats_tx.try_send(*stats) {
                    if e.is_disconnected() {
                        warn!("Failed to publish stats snapshot: {e:?}");
                    }
                }
            }
        }
    }
}
