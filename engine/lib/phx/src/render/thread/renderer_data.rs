use crossbeam::channel::{Receiver, Sender};

use crate::render::{
    ClipManager, DrawState, PrimitiveBuilder, RenderBatch, RenderCommand, RenderStateIntern,
    RenderTargetStack, ResourceId, Shader, ShaderErrorQueue, ShaderVarMap, ShaderWatcherInner,
    VpStack,
};

pub struct RendererData {
    /// Counter for generating unique ResourceIds. Only ever touched through
    /// `&mut self`, so a plain integer is enough - no atomic needed.
    pub(super) next_resource_id: u64,
    /// Producer end of the destroy queue, cloned into every `ResourceHandle`
    /// so a resource's `Drop` can enqueue its id without a `&mut Renderer`.
    pub(super) destroy_tx: Sender<ResourceId>,
    /// Consumer end, owned solely by this `Renderer` and drained once per
    /// frame in `end_frame_triple_buffered`.
    pub(super) destroy_rx: Receiver<ResourceId>,
    /// Command buffer used by the batch API (`begin_batch`/`flush_batch`)
    pub(super) command_buffer: Vec<RenderCommand>,
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
    /// Shader compile/reload error queue, for the hot-reload error overlay
    pub(crate) shader_errors: ShaderErrorQueue,
    /// File-watcher state for shader hot-reload; `None` until `ShaderWatcher::Init` runs
    pub(crate) shader_watcher: Option<ShaderWatcherInner>,
    /// Lazily-created shader for `Mesh::compute_ao` (was `static mut SHADER`)
    pub(crate) ao_shader: Option<Shader>,
    /// Lazily-created shader for `Mesh::compute_occlusion` (was `static mut SHADER`)
    pub(crate) occlusion_shader: Option<Shader>,
    /// Lazily-created shader for `TexCube::gen_ir_map` (was `static mut SHADER`)
    pub(crate) irmap_shader: Option<Shader>,
}
