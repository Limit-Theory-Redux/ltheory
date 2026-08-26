use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::render::thread::{CommandExecutor, CommandReply};
use crate::render::{RenderCommand, RenderStats, ShaderReloadResult};
use crate::window::{WindowActiveGlContext, WindowGlContext};

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
    /// Channel to send shader reload results back to main thread
    shader_result_tx: Sender<ShaderReloadResult>,
    /// Channel to return GL context to main thread on shutdown
    context_tx: Sender<Option<WindowGlContext>>,
    /// Channel to publish a stats snapshot to the main thread on every frame
    stats_tx: Sender<RenderStats>,
    running: Arc<AtomicBool>,
    executor: CommandExecutor,
}

impl RenderThread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        pacing_fence_tx: Sender<u64>,
        shader_result_tx: Sender<ShaderReloadResult>,
        context_tx: Sender<Option<WindowGlContext>>,
        stats_tx: Sender<RenderStats>,
        running: Arc<AtomicBool>,
        gl_context: Option<WindowActiveGlContext>,
        category_timing: Arc<AtomicBool>,
    ) -> Self {
        Self {
            command_rx,
            fence_tx,
            pacing_fence_tx,
            shader_result_tx,
            context_tx,
            stats_tx,
            running,
            executor: CommandExecutor::new_with_timing(gl_context, category_timing),
        }
    }

    /// Main render loop
    pub fn run(&mut self) {
        info!("Render thread started");

        // Only initialize GL resources if we have a valid context
        if self.executor.has_gl_context() {
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

                    if recv_wait_us >= STARVATION_THRESHOLD_US {
                        self.executor.this_frame_stats.recv_wait_us_last_frame += recv_wait_us;
                        self.executor.this_frame_stats.recv_wait_count_last_frame += 1;
                    }

                    let reply = self.executor.execute(cmd);
                    self.dispatch(reply);
                }
                Err(_) => {
                    debug!("Command channel closed, render thread exiting");
                    break;
                }
            }
        }

        let context = self.executor.cleanup();
        if let Err(e) = self.context_tx.send(context) {
            error!("Failed to signal main thread on shutdown: {e:?}");
        }

        info!("Render thread stopped. Stats: {:?}", self.executor.stats());
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
            CommandReply::ShaderReload(result) => {
                if let Err(e) = self.shader_result_tx.send(result) {
                    error!("Failed to send shader reload result: {e:?}");
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
