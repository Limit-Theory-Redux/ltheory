use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};
use tracing::{debug, info, warn};

use crate::render::thread::CommandReply;
use crate::render::thread::command_executor_wgpu::WgpuCommandExecutor;
use crate::render::{PassCommands, RenderCommand, RenderStats, ReturnedChunk};
use crate::window::WgpuStartupBundle;

/// How long the loop waits for a command before polling the device. The
/// device's callbacks (pacing fences, readbacks) only run when it is polled,
/// and the main thread may be blocked on exactly one of them.
const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(1);

/// Drives a [`WgpuCommandExecutor`] on a dedicated thread.
///
/// This type owns only the plumbing: it pulls commands off a channel, hands
/// them to the executor, and forwards whatever the executor answers back to
/// the main thread. All GPU work and all GPU state live in the executor.
pub struct RenderThread {
    command_rx: Receiver<RenderCommand>,
    fence_tx: Sender<u64>,
    /// Reply channel for `RenderCommand::PacingFence`, kept separate from
    /// `fence_tx` so frame-pacing fences and `sync_intern`'s blocking
    /// round-trip fences can never be consumed by the wrong consumer (see
    /// `RenderCommand::PacingFence`'s docs).
    pacing_fence_tx: Sender<u64>,
    /// Channel to publish a stats snapshot to the main thread on every frame
    stats_tx: Sender<RenderStats>,
    /// Channel returning uploaded ring memory to the main thread
    chunk_return_tx: Sender<ReturnedChunk>,
    running: Arc<AtomicBool>,
    /// `None` only for a headless renderer (unit tests without a window):
    /// commands are then answered by [`headless_execute`].
    executor: Option<WgpuCommandExecutor>,
}

impl RenderThread {
    /// The executor is built from the surface bundle (device, queue, surface
    /// and initial size all move here).
    pub fn new(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        pacing_fence_tx: Sender<u64>,
        stats_tx: Sender<RenderStats>,
        chunk_return_tx: Sender<ReturnedChunk>,
        running: Arc<AtomicBool>,
        bundle: Option<WgpuStartupBundle>,
    ) -> Self {
        let executor = bundle.map(|bundle| {
            let (width, height) = (bundle.surface_config.width, bundle.surface_config.height);
            let mut executor = WgpuCommandExecutor::with_device(bundle.device, bundle.queue)
                .with_surface(bundle.surface, bundle.surface_config, width, height);
            // Frame pacing follows the GPU: the executor answers a pacing
            // fence when `on_submitted_work_done` fires.
            executor.set_pacing_sender(pacing_fence_tx.clone());
            executor
        });
        Self {
            command_rx,
            fence_tx,
            pacing_fence_tx,
            stats_tx,
            chunk_return_tx,
            running,
            executor,
        }
    }

    /// Main render loop
    pub fn run(&mut self) {
        info!("Render thread started");
        if self.executor.is_none() {
            warn!("Render thread running without a GPU device - commands will be no-ops");
        }

        while self.running.load(Ordering::Relaxed) {
            // Measure render-thread idle time: how long we block waiting for
            // commands (the producer-starvation gap). A fast-path recv on a
            // populated channel returns in a few µs; a genuinely blocked recv
            // waits hundreds of µs+. Only waits above the threshold count as
            // starvation, so fast-path scheduling noise stays out.
            const STARVATION_THRESHOLD_US: u64 = 20;

            let recv_start = std::time::Instant::now();
            let Some(cmd) = self.next_command() else {
                debug!("Command channel closed, render thread exiting");
                break;
            };
            let recv_wait_us = recv_start.elapsed().as_micros() as u64;

            if matches!(cmd, RenderCommand::Shutdown) {
                info!("Render thread received shutdown command");
                break;
            }

            let Some(executor) = self.executor.as_mut() else {
                let (reply, returned) = headless_execute(cmd);
                self.dispatch(reply);
                for chunk in returned {
                    let _ = self.chunk_return_tx.send(chunk);
                }
                continue;
            };
            if recv_wait_us >= STARVATION_THRESHOLD_US {
                executor.note_recv_wait(recv_wait_us);
            }
            let reply = executor.execute(cmd);
            // Ring memory the command finished uploading goes back to the
            // main thread for reuse.
            let returned = executor.take_returned_chunks();
            self.dispatch(reply);
            for chunk in returned {
                let _ = self.chunk_return_tx.send(chunk);
            }
        }

        if let Some(executor) = &self.executor {
            info!("Render thread stopped. Stats: {:?}", executor.stats());
        }
    }

    /// Wait for the next command, polling the device while the channel is
    /// idle. `None` once the channel is closed or the renderer stops running.
    fn next_command(&self) -> Option<RenderCommand> {
        let Some(executor) = self.executor.as_ref() else {
            return self.command_rx.recv().ok();
        };
        loop {
            match self.command_rx.recv_timeout(IDLE_POLL_INTERVAL) {
                Ok(cmd) => return Some(cmd),
                Err(RecvTimeoutError::Timeout) => {
                    executor.poll();
                    if !self.running.load(Ordering::Relaxed) {
                        return None;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return None,
            }
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

/// What a renderer without a GPU device answers: nothing is drawn, but every
/// caller blocked on a reply is released (fences, shader creation, readbacks)
/// and ring memory goes back to the main thread as it would after an upload.
fn headless_execute(cmd: RenderCommand) -> (CommandReply, Vec<ReturnedChunk>) {
    let mut returned = Vec::new();
    let reply = match cmd {
        RenderCommand::Fence { fence_id } => CommandReply::Fence(fence_id),
        RenderCommand::PacingFence { fence_id } => CommandReply::PacingFence(fence_id),
        RenderCommand::CreateShader { reply_tx, .. } => {
            let _ = reply_tx.send(Ok(Vec::new()));
            CommandReply::None
        }
        RenderCommand::ReadTextureSync { reply_tx, .. } => {
            let _ = reply_tx.send(Vec::new());
            CommandReply::None
        }
        RenderCommand::PassCommands(commands) => {
            let PassCommands {
                uniforms, vertices, ..
            } = *commands;
            for (vertex, chunks) in [(false, uniforms), (true, vertices)] {
                returned.extend(chunks.into_iter().map(|chunk| ReturnedChunk {
                    vertex,
                    bytes: chunk.bytes,
                }));
            }
            CommandReply::None
        }
        _ => CommandReply::None,
    };
    (reply, returned)
}

/// The `key=value` lines `Renderer::backend_info` returns.
pub(super) fn wgpu_backend_info(
    adapter: &wgpu::Adapter,
    config: &wgpu::SurfaceConfiguration,
) -> String {
    let i = adapter.get_info();
    format!(
        "backend=wgpu\nwgpu_backend={:?}\nadapter={}\ndevice_type={:?}\ndriver={}\ndriver_info={}\nvendor_id=0x{:04x}\ndevice_id=0x{:04x}\nsurface_format={:?}\npresent_mode={:?}\n",
        i.backend,
        i.name,
        i.device_type,
        i.driver,
        i.driver_info,
        i.vendor,
        i.device,
        config.format,
        config.present_mode
    )
}
