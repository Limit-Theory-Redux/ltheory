use crossbeam::channel::{Receiver, Sender};

use crate::render::{
    ClipManager, DrawState, PrimitiveBuilder, RenderBatch, RenderCommand, RenderStateIntern,
    RenderTargetStack, ResourceId, Shader, ShaderErrorQueue, ShaderVarMap, ShaderWatcherInner,
    VpStack,
};

pub struct RendererData {
    /// Counter for generating unique ResourceIds. Only ever touched through
    /// `&mut self`, so a plain integer is enough - no atomic needed.
    pub next_resource_id: u64,
    /// Producer end of the destroy queue, cloned into every `ResourceHandle`
    /// so a resource's `Drop` can enqueue its id without a `&mut Renderer`.
    pub destroy_tx: Sender<ResourceId>,
    /// Consumer end, owned solely by this `Renderer` and drained once per
    /// frame in `end_frame_triple_buffered`.
    pub destroy_rx: Receiver<ResourceId>,
    /// Command buffer used by the batch API (`begin_batch`/`flush_batch`)
    pub command_buffer: Vec<RenderCommand>,
    /// Active render batch
    pub active_batch: Option<RenderBatch>,
    /// Viewport stack (was `thread_local! VP_STACK` in viewport.rs)
    pub viewport: VpStack,
    /// Framebuffer attachment bookkeeping (was `thread_local! FBO_STACK` in
    /// render_target.rs)
    pub render_target: RenderTargetStack,
    /// Clip-rect stack (was `thread_local! CLIP_MANAGER` in clip_rect.rs)
    pub clip_rect: ClipManager,
    /// GL state stack (was `thread_local! RENDER_STATE` in render_state.rs)
    pub render_state: RenderStateIntern,
    /// Immediate-mode vertex accumulator (was `Draw`'s owned `PrimitiveBuilder`)
    pub imm: PrimitiveBuilder,
    /// `Draw`'s CPU-side alpha/color stack (was static via `Draw::inst()`)
    pub draw_state: DrawState,
    /// Shader auto-var stack (was `static OnceLock<Mutex<ShaderVar>>`)
    pub shader_vars: ShaderVarMap,
    /// Shader compile/reload error queue, for the hot-reload error overlay
    pub shader_errors: ShaderErrorQueue,
    /// File-watcher state for shader hot-reload; `None` until `ShaderWatcher::Init` runs
    pub shader_watcher: Option<ShaderWatcherInner>,
    /// Lazily-created shader for `Mesh::compute_ao` (was `static mut SHADER`)
    pub ao_shader: Option<Shader>,
    /// Lazily-created shader for `Mesh::compute_occlusion` (was `static mut SHADER`)
    pub occlusion_shader: Option<Shader>,
    /// Lazily-created shader for `TexCube::gen_ir_map` (was `static mut SHADER`)
    pub irmap_shader: Option<Shader>,
}
