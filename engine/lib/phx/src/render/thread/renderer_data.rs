use crossbeam::channel::{Receiver, Sender};

use crate::render::{
    CameraState, ClipManager, DrawState, Environment, PassEncoder, PassState, PipelineCache,
    PrimitiveBuilder, RenderBatch, RenderStateIntern, ResourceId, ReturnedChunk,
    RingOffset, SamplerCache, ScissorUpdate, Shader, ShaderErrorQueue, ShaderWatcherInner,
    UniformRing, VertexRing, ViewBlock,
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
    /// Active render batch
    pub active_batch: Option<RenderBatch>,
    /// The open render pass (one at a time) and the size of the last target.
    pub pass: PassState,
    /// Records the open pass's commands until they are flushed.
    pub encoder: PassEncoder,
    /// Uniform staging for per-pass and per-draw blocks.
    pub ring: UniformRing,
    /// Staging for instance data and instanced index lists.
    pub vertex_ring: VertexRing,
    /// Frames ended so far; selects the ring slot.
    pub frame_index: u64,
    /// Pipeline hash cache.
    pub pipelines: PipelineCache,
    /// Sampler hash cache (presets pre-registered).
    pub samplers: SamplerCache,
    /// Next `BindGroupId`.
    pub next_bind_group: u32,
    /// The `ViewBlock` uploaded last, where, and in which frame, so passes
    /// with an identical block reuse the upload.
    pub last_view: Option<(ViewBlock, RingOffset, u64)>,
    /// Camera of the next passes (`Renderer:setCamera`).
    pub camera: CameraState,
    /// Group-0 environment maps (`Renderer:setEnvironment`).
    pub environment: Environment,
    /// Clip-rect stack (was `thread_local! CLIP_MANAGER` in clip_rect.rs)
    pub clip_rect: ClipManager,
    /// The scissor update last sent to the GPU; the GL scissor is global
    /// state, so `ClipRect` compares against this, not against the pass.
    pub clip_emitted: Option<ScissorUpdate>,
    /// GL state stack (was `thread_local! RENDER_STATE` in render_state.rs)
    pub render_state: RenderStateIntern,
    /// Immediate-mode vertex accumulator (was `Draw`'s owned `PrimitiveBuilder`)
    pub imm: PrimitiveBuilder,
    /// `Draw`'s CPU-side alpha/color stack (was static via `Draw::inst()`)
    pub draw_state: DrawState,
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

impl RendererData {
    /// Ring memory the executor finished uploading: back into its pool.
    pub fn recycle_chunk(&mut self, chunk: ReturnedChunk) {
        if chunk.vertex {
            self.vertex_ring.recycle(chunk.bytes);
        } else {
            self.ring.recycle(chunk.bytes);
        }
    }

    pub fn new(destroy_tx: Sender<ResourceId>, destroy_rx: Receiver<ResourceId>) -> Self {
        Self {
            next_resource_id: 1,
            destroy_tx,
            destroy_rx,
            active_batch: None,
            pass: PassState::default(),
            encoder: PassEncoder::new(),
            ring: UniformRing::new(),
            vertex_ring: VertexRing::new(),
            frame_index: 0,
            pipelines: PipelineCache::new(),
            samplers: SamplerCache::new(),
            next_bind_group: 0,
            last_view: None,
            camera: CameraState::default(),
            environment: Environment::default(),
            clip_rect: ClipManager::new(),
            clip_emitted: None,
            render_state: RenderStateIntern::new(),
            imm: PrimitiveBuilder::new(),
            draw_state: DrawState::new(),
            shader_errors: ShaderErrorQueue::new(),
            shader_watcher: None,
            ao_shader: None,
            occlusion_shader: None,
            irmap_shader: None,
        }
    }
}
