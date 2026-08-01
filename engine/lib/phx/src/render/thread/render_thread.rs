use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::render::thread::{CommandExecutor, CommandReply};
use crate::render::{RenderCommand, ShaderReloadResult, SharedRenderStats};
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
    /// Channel to send shader reload results back to main thread
    shader_result_tx: Sender<ShaderReloadResult>,
    /// Channel to return GL context to main thread on shutdown
    context_tx: Sender<Option<WindowGlContext>>,
    running: Arc<AtomicBool>,
    executor: CommandExecutor,
}

impl RenderThread {
    pub fn new(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        shader_result_tx: Sender<ShaderReloadResult>,
        context_tx: Sender<Option<WindowGlContext>>,
        running: Arc<AtomicBool>,
        shared_stats: Arc<SharedRenderStats>,
        gl_context: Option<WindowActiveGlContext>,
    ) -> Self {
        Self {
            command_rx,
            fence_tx,
            shader_result_tx,
            context_tx,
            running,
            executor: CommandExecutor::new(shared_stats, gl_context),
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
            match self.command_rx.recv() {
                Ok(cmd) => {
                    if matches!(cmd, RenderCommand::Shutdown) {
                        info!("Render thread received shutdown command");
                        break;
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
            CommandReply::ShaderReload(result) => {
                if let Err(e) = self.shader_result_tx.send(result) {
                    error!("Failed to send shader reload result: {e:?}");
                }
            }
        }
    }
}
