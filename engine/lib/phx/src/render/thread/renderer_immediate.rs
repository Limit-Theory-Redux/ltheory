use crossbeam::channel::unbounded;
use tracing::{error, info, warn};

use crate::render::thread::{CommandExecutor, CommandReply, RendererData, process_batch_intern};
use crate::render::{
    ClipManager, DrawState, PrimitiveBuilder, RenderCommand, RenderStateIntern, RenderStats,
    RenderTargetStack, RenederThreadError, ResourceHandle, ResourceId, ShaderReloadResult,
    ShaderVarMap, VpStack,
};
use crate::window::WindowGlContext;

pub struct Renderer {
    /// Executes commands inline on whichever thread calls `submit`.
    executor: CommandExecutor,
    /// Generic renderer data
    pub(crate) data: RendererData,
}

impl Renderer {
    pub fn start(context: WindowGlContext) -> Result<Self, RenederThreadError> {
        let gl_context = match context.make_current() {
            Ok(ctx) => {
                info!("GL context made current");
                Some(ctx)
            }
            Err(e) => {
                error!("Failed to make GL context current: {}", e);
                None
            }
        };

        let mut executor = CommandExecutor::new(gl_context);
        if executor.has_gl_context() {
            executor.init_gl();
        } else {
            warn!("Renderer running without GL context - commands will be no-ops");
        }

        info!("Renderer started (immediate mode)");

        // Unbounded: `ResourceHandle::drop` must never block or fail.
        let (destroy_tx, destroy_rx) = unbounded();

        Ok(Self {
            executor,
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
                ao_shader: None,
                occlusion_shader: None,
                irmap_shader: None,
            },
        })
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
                ao_shader: None,
                occlusion_shader: None,
                irmap_shader: None,
            },
        }
    }

    pub fn stop(mut self) -> Option<WindowGlContext> {
        info!("Stopping renderer (immediate mode)");
        self.executor.cleanup()
    }

    /// Execute a command inline.
    pub fn submit(&mut self, cmd: RenderCommand) {
        self.executor.execute(cmd);
    }

    /// Execute a command inline. Always succeeds - immediate mode has no
    /// channel to back up, so there is nothing to drop.
    pub fn try_submit(&mut self, cmd: RenderCommand) -> bool {
        self.submit(cmd);
        true
    }

    /// Begin a new frame
    pub(in crate::render::thread) fn begin_frame_intern(&mut self) {
        self.data.command_buffer.clear();
    }

    /// Run every buffered command (from the batch API) inline.
    pub(in crate::render::thread) fn flush_intern(&mut self) {
        for cmd in self.data.command_buffer.drain(..) {
            self.executor.execute(cmd);
        }
    }

    /// Immediate mode has nothing to wait for: by the time `submit` returns,
    /// the command has already executed.
    pub(in crate::render::thread) fn sync_intern(&mut self) -> bool {
        true
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
        for id in ids {
            self.submit(RenderCommand::DestroyResource { id });
        }
    }

    /// Immediate mode has no frame queue to pace against - just swap.
    pub fn end_frame_triple_buffered(&mut self) {
        self.drain_destroy_queue();
        self.submit(RenderCommand::SwapBuffers);
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
        self.executor.stats_snapshot().draw_calls
    }

    /// Get total state changes since start
    pub fn get_state_changes(&mut self) -> u64 {
        self.executor.stats_snapshot().state_changes
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
        self.executor.stats_snapshot().commands_last_frame
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&mut self) -> u64 {
        self.executor.stats_snapshot().draw_calls_last_frame
    }

    /// Always zero - immediate mode never blocks waiting on a render thread.
    pub fn get_main_thread_wait_us(&self) -> u64 {
        0
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&mut self) -> u64 {
        self.executor.stats_snapshot().texture_binds_skipped
    }

    /// Reload a shader inline and return the result directly - no channel
    /// round-trip needed since the executor answers synchronously.
    pub fn reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> ShaderReloadResult {
        let reply = self
            .executor
            .cmd_reload_shader(shader_key, vertex_src, fragment_src);
        // execute(RenderCommand::ReloadShader {
        //     shader_key: shader_key.to_string(),
        //     vertex_src: vertex_src.to_string(),
        //     fragment_src: fragment_src.to_string(),
        // });

        match reply {
            CommandReply::ShaderReload(result) => result,
            _ => ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Executor returned no shader reload result".to_string()),
                program: 0,
            },
        }
    }

    pub fn process_batch(&mut self) {
        process_batch_intern(&mut self.data.active_batch, &mut self.data.command_buffer);
    }

    /// Immediate mode has nothing pending to poll for - `stop()` already
    /// returns the context synchronously.
    pub fn take_returned_context(&self) -> Option<WindowGlContext> {
        None
    }
}
