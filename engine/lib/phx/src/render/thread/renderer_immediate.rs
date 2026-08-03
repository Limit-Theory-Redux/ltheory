use std::sync::atomic::{AtomicU64, Ordering};

use tracing::{error, info, warn};

use crate::render::thread::{CommandExecutor, CommandReply, process_batch_intern};
use crate::render::{
    ClipManager, DrawState, RenderBatch, RenderStateIntern, RenderCommand, PrimitiveBuilder, Shader, ShaderVarMap, RenderStats, RenderTargetStack, RenederThreadError,
    ResourceId, ShaderReloadResult, VpStack,
};
use crate::window::WindowGlContext;

pub struct Renderer {
    /// Executes commands inline on whichever thread calls `submit`.
    executor: CommandExecutor,
    /// Global counter for generating unique ResourceIds
    next_resource_id: AtomicU64,
    /// Command buffer used by the batch API (`begin_batch`/`flush_batch`)
    command_buffer: Vec<RenderCommand>,
    /// Active render batch
    pub(super) active_batch: Option<RenderBatch>,
    /// Viewport stack (was `thread_local! VP_STACK` in viewport.rs)
    pub(crate) viewport: VpStack,
    /// Framebuffer attachment bookkeeping (was `thread_local! FBO_STACK` in
    /// render_target.rs)
    pub(crate) render_target: RenderTargetStack,
    /// Clip-rect stack (was `thread_local! CLIP_MANAGER` in clip_rect.rs)
    pub(crate) clip_rect: ClipManager,
    /// GL state stack (was `thread_local! RENDER_STATE` in render_state.rs)
    pub(crate) render_state: RenderStateIntern,
    /// Immediate-mode vertex accumulator (was `Draw`'s owned `PrimitiveBuilder`)
    pub(crate) imm: PrimitiveBuilder,
    /// `Draw`'s CPU-side alpha/color stack (was static via `Draw::inst()`)
    pub(crate) draw_state: DrawState,
    /// Shader auto-var stack (was `static OnceLock<Mutex<ShaderVar>>`)
    pub(crate) shader_vars: ShaderVarMap,
    /// Lazily-created shader for `Mesh::compute_ao` (was `static mut SHADER`)
    pub(crate) ao_shader: Option<Shader>,
    /// Lazily-created shader for `Mesh::compute_occlusion` (was `static mut SHADER`)
    pub(crate) occlusion_shader: Option<Shader>,
    /// Lazily-created shader for `TexCube::gen_ir_map` (was `static mut SHADER`)
    pub(crate) irmap_shader: Option<Shader>,
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

        Ok(Self {
            executor,
            next_resource_id: AtomicU64::new(1),
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
        })
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
        self.command_buffer.clear();
    }

    /// Run every buffered command (from the batch API) inline.
    pub(in crate::render::thread) fn flush_intern(&mut self) {
        for cmd in self.command_buffer.drain(..) {
            self.executor.execute(cmd);
        }
    }

    /// Immediate mode has nothing to wait for: by the time `submit` returns,
    /// the command has already executed.
    pub(in crate::render::thread) fn sync_intern(&mut self) -> bool {
        true
    }

    /// Generate a new unique resource ID
    pub fn next_resource_id(&mut self) -> ResourceId {
        ResourceId(self.next_resource_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Immediate mode has no frame queue to pace against - just swap.
    pub fn end_frame_triple_buffered(&mut self) {
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
        let reply = self.executor.execute(RenderCommand::ReloadShader {
            shader_key: shader_key.to_string(),
            vertex_src: vertex_src.to_string(),
            fragment_src: fragment_src.to_string(),
        });

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
        process_batch_intern(&mut self.active_batch, &mut self.command_buffer);
    }

    /// Immediate mode has nothing pending to poll for - `stop()` already
    /// returns the context synchronously.
    pub fn take_returned_context(&self) -> Option<WindowGlContext> {
        None
    }
}
