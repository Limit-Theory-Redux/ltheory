//! wgpu backend of the render-thread command executor (render API v2, S11).
//!
//! The command stream is the one the GL executor runs, and the executor maps
//! it onto wgpu the way the API was shaped for:
//!
//! * **One `CommandEncoder` per frame.** Work is recorded into it and
//!   submitted at `SwapBuffers`, at an explicit `Flush`, before a readback and
//!   before anything that writes a resource through the queue (`write_texture`
//!   and `write_buffer` are ordered before a whole submission, so recorded
//!   passes that use the resource must be submitted first). See `frame.rs`.
//! * **One `wgpu::RenderPass` per `BeginRenderPass`/`EndRenderPass`** with the
//!   real load and store ops (`pass.rs`). A pass that a flush point interrupts
//!   (a `CopyTexture` or mip generation in the middle of it) is closed and
//!   reopened with `LoadOp::Load` before its next draw.
//! * **Pipelines** are built from the `PipelineDesc` plus what the pass and
//!   the draw say: the attachment formats of the pass and the vertex layout
//!   of the draw command key a variant of each `PipelineId`.
//! * **Bind groups follow the 4-group model.** `#group N` is `set N`.
//!   Group 0 (`ViewBlock`) and group 2 (the draw/params block) bind a ring
//!   buffer with a dynamic offset; group 1 binds a material arena slice and
//!   textures; group 3 is the pass inputs. Bind groups are created from the
//!   state the commands left behind and cached by content (`bind.rs`).
//! * **Shaders** are the engine's GLSL 330 through naga with a handful of
//!   mechanical adaptations (`shader.rs`).
//!
//! # Coordinate conventions (the one place they are decided)
//!
//! GL and wgpu disagree about three things, and this backend resolves all of
//! them in the vertex stage and at present, once:
//!
//! 1. **Framebuffer origin.** GL's framebuffer row 0 is at NDC `y = -1`
//!    (bottom); wgpu's is at NDC `y = +1` (top). Textures are indexed the same
//!    way in both (row 0 is uv `v = 0`), so a texture rendered by the engine
//!    would come out upside down. Every vertex shader therefore negates
//!    `gl_Position.y` (the wrapper in `shader.rs`), and **all** targets,
//!    including the window, are GL-convention: row 0 is the bottom. Front
//!    faces flip with the mirrored y, so pipelines use `FrontFace::Cw`.
//!    Viewports, scissors and `gl_FragCoord` (both count rows from the same
//!    origin now) need no conversion.
//! 2. **The window.** The backbuffer is an ordinary GL-convention texture
//!    (`RGBA8`, plus a depth buffer); `SwapBuffers` copies it to the swapchain
//!    image upside down with a blit pass, so the window shows what GL shows
//!    and `ReadSource::Backbuffer` reads the same rows as GL.
//! 3. **Clip depth.** GL clips z to `-w..w`, wgpu to `0..w`: the same wrapper
//!    maps `z' = (z + w) / 2`. `gl_FragDepth` already is a window-space
//!    `0..1` value in both, so logarithmic depth works unchanged.
//!
//! Surface format is the linear twin of the preferred swapchain format (the
//! GL default framebuffer is not sRGB).

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam::channel::Sender;
use tracing::{error, warn};

use crate::render::thread::ExecutorStats;
use crate::render::{
    BindEntry, BindGroupId, BlendMode, BlockLayout, BufferId, CommandReply, CompareFn, CullFace,
    ImmLayout, InstanceData, LoadOp, MipFilter, PassCmd, PassCommands, PipelineDesc, PipelineId,
    RenderCommand, RenderPassDesc, RenderStats, ResourceId, ReturnedChunk, SamplerDesc,
    SamplerFilter, SamplerId, ShaderLayout, TexDesc, TexFormat, TexRegion, TexView, TexWrapMode,
    Topology, VertexFormat, ViewDim,
};
use crate::window::PresentMode;

mod bind;
mod formats;
mod frame;
mod pass;
mod readback;
mod shader;
mod tex;

use bind::{CachedBg, Defaults, GroupLayout, GroupLayoutKey};
use formats::*;
use pass::{PassRt, Quad, VariantKey, ViewKey};
use shader::VertexInput;
#[cfg(test)]
pub(crate) use shader::{adapt_single_stage, parse_adapted};

/// A compiled shader program: the two modules, the layout it was declared
/// with and what the executor reflected from it.
pub(super) struct WgpuShader {
    vs: wgpu::ShaderModule,
    fs: wgpu::ShaderModule,
    /// The adapted fragment source and its outputs: the clamped fragment
    /// variants are compiled from them on demand.
    fs_code: String,
    fs_named_outputs: Vec<shader::FsOutput>,
    vs_inputs: Vec<VertexInput>,
    fs_outputs: Vec<u32>,
    groups: [Arc<GroupLayout>; 4],
    pipeline_layout: wgpu::PipelineLayout,
}

pub(super) struct WgpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_format: VertexFormat,
}

/// A resource by id. The kind of a texture is `desc.dim`.
pub(super) enum WgpuResource {
    Shader(Box<WgpuShader>),
    Texture {
        texture: wgpu::Texture,
        desc: TexDesc,
    },
    Mesh(WgpuMesh),
}

/// A bind group made with `CreateBindGroup`: what `SetBindGroup` binds.
pub(super) struct StoredBindGroup {
    group: u8,
    entries: Vec<BindEntry>,
}

/// wgpu backend for [`RenderCommand`] execution.
///
/// Owns the device, queue and surface and every wgpu object; it has no idea
/// whether it runs on the render thread or inline.
pub struct WgpuCommandExecutor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    features: wgpu::Features,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    surface_size: (u32, u32),

    // === Resources ===
    resources: HashMap<ResourceId, WgpuResource>,
    /// How many times each shader or texture was created under its id. Part
    /// of every cache key that holds a reference to one.
    generations: HashMap<ResourceId, u32>,
    pipeline_descs: HashMap<PipelineId, PipelineDesc>,
    samplers: HashMap<SamplerId, wgpu::Sampler>,
    bind_groups: HashMap<BindGroupId, StoredBindGroup>,
    buffers: HashMap<BufferId, wgpu::Buffer>,

    // === Caches ===
    group_layouts: HashMap<GroupLayoutKey, Arc<GroupLayout>>,
    next_layout_id: u32,
    variants: HashMap<VariantKey, wgpu::RenderPipeline>,
    /// Fragment modules with clamped outputs by `(shader, generation, mask)`.
    fs_clamped: HashMap<(ResourceId, u32, u32), wgpu::ShaderModule>,
    bg_cache: HashMap<Vec<u64>, CachedBg>,
    next_bg_serial: u64,
    views: HashMap<ViewKey, wgpu::TextureView>,
    defaults: Option<Defaults>,
    quad: Option<Quad>,
    warned: std::collections::HashSet<String>,

    // === Frame state (frame.rs) ===
    encoder: Option<wgpu::CommandEncoder>,
    /// Passes or copies were recorded into `encoder` since the last submit.
    recorded: bool,
    pass: Option<PassRt>,
    frame_slot: usize,
    frame_index: u64,
    /// Uniform and vertex ring buffers by `[slot][chunk]`.
    ring_uniform: [Vec<wgpu::Buffer>; crate::render::MAX_FRAMES_IN_FLIGHT],
    ring_vertex: [Vec<wgpu::Buffer>; crate::render::MAX_FRAMES_IN_FLIGHT],
    returned: Vec<ReturnedChunk>,
    backbuffer: Option<frame::Backbuffer>,
    present: Option<frame::PresentBlit>,
    /// Set by `on_submitted_work_done` once the GPU finished a slot's frame.
    slot_done: [Option<Arc<AtomicBool>>; crate::render::MAX_FRAMES_IN_FLIGHT],
    pacing_tx: Option<Sender<u64>>,
    pending_reads: Vec<readback::WgpuPendingRead>,
    mip_blit: tex::MipBlit,

    // === Stats ===
    stats: ExecutorStats,
    last_stats: RenderStats,
    frame_counters: FrameCounters,
    /// End of the previous frame's present: a frame's render time is measured from here.
    frame_start: Instant,
}

/// Per-frame counters behind `RenderStats`.
#[derive(Default)]
pub(super) struct FrameCounters {
    commands: u64,
    draw_mesh: u64,
    draw_imm: u64,
    draw_instanced: u64,
    imm_vertices: u64,
    instance_items: u64,
    vertices: u64,
    state_changes: u64,
    pipeline_binds: u64,
    pipeline_redundant: u64,
    submits: u64,
    passes: u64,
    bind_group_switches: u64,
    recv_wait_us: u64,
    recv_wait_count: u64,
}

impl WgpuCommandExecutor {
    /// An executor for `device` and `queue`. Errors the device reports are
    /// logged; with `LTHEORY_WGPU_FATAL=1` the first one panics (what the
    /// validation runs use, so nothing is silently ignored).
    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let fatal = std::env::var("LTHEORY_WGPU_FATAL").is_ok_and(|v| v != "0" && !v.is_empty());
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            if fatal {
                panic!("wgpu error (LTHEORY_WGPU_FATAL): {error}");
            }
            error!("wgpu error: {error}");
        }));
        device.set_device_lost_callback(|reason, message| {
            error!("wgpu device lost ({reason:?}): {message}");
        });
        let features = device.features();
        Self {
            device,
            queue,
            features,
            surface: None,
            surface_config: None,
            surface_size: (1, 1),
            resources: HashMap::new(),
            generations: HashMap::new(),
            pipeline_descs: HashMap::new(),
            samplers: HashMap::new(),
            bind_groups: HashMap::new(),
            buffers: HashMap::new(),
            group_layouts: HashMap::new(),
            next_layout_id: 1,
            variants: HashMap::new(),
            fs_clamped: HashMap::new(),
            bg_cache: HashMap::new(),
            next_bg_serial: 1,
            views: HashMap::new(),
            defaults: None,
            quad: None,
            warned: Default::default(),
            encoder: None,
            recorded: false,
            pass: None,
            frame_slot: 0,
            frame_index: 0,
            ring_uniform: Default::default(),
            ring_vertex: Default::default(),
            returned: Vec::new(),
            backbuffer: None,
            present: None,
            slot_done: Default::default(),
            pacing_tx: None,
            pending_reads: Vec::new(),
            mip_blit: tex::MipBlit::default(),
            stats: ExecutorStats::default(),
            last_stats: RenderStats::default(),
            frame_counters: FrameCounters::default(),
            frame_start: Instant::now(),
        }
    }

    /// Attach the swapchain (render-thread startup path). It is reconfigured
    /// on `Resize`.
    pub fn with_surface(
        mut self,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> Self {
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.surface_size = (width.max(1), height.max(1));
        self
    }

    /// Where `PacingFence` completions go once the GPU has really finished
    /// the work before them. Without a sender they reply immediately.
    pub fn set_pacing_sender(&mut self, tx: Sender<u64>) {
        self.pacing_tx = Some(tx);
    }

    /// Accepted for interface parity with the GL executor (the stats
    /// dashboard's per-category timing is not collected here).
    pub fn set_category_timing(&self, _timing: Arc<AtomicBool>) {}

    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// Run the device's callbacks (fence completions, buffer maps). The render
    /// thread calls this whenever it has nothing to execute.
    pub fn poll(&self) {
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    /// Ring memory the executor finished uploading, for the main thread to
    /// reuse.
    pub fn take_returned_chunks(&mut self) -> Vec<ReturnedChunk> {
        std::mem::take(&mut self.returned)
    }

    /// Time the render thread waited for a command (producer starvation), for the stats.
    pub fn note_recv_wait(&mut self, us: u64) {
        self.frame_counters.recv_wait_us += us;
        self.frame_counters.recv_wait_count += 1;
    }

    /// Log `what` once (a missing texture, an unsupported feature).
    fn warn_once(&mut self, what: String) {
        if self.warned.insert(what.clone()) {
            warn!("wgpu: {what}");
        }
    }

    // =====================================================================
    // Resource commands
    // =====================================================================

    pub(super) fn generation_of(&self, id: ResourceId) -> u32 {
        self.generations.get(&id).copied().unwrap_or(0)
    }

    fn bump_generation(&mut self, id: ResourceId) {
        *self.generations.entry(id).or_insert(0) += 1;
    }

    /// Storage format of an engine texture format, and its bytes per texel.
    /// 32-bit float textures are native when the adapter filters and blends
    /// them (`wanted_features`); otherwise `R32F` and `RGBA32F` live in
    /// 16-bit float textures (lossy, but filterable).
    pub(super) fn storage_format(&self, format: TexFormat) -> (wgpu::TextureFormat, u32) {
        use wgpu::TextureFormat as F;
        let native32 = self
            .features
            .contains(wgpu::Features::FLOAT32_FILTERABLE | wgpu::Features::FLOAT32_BLENDABLE);
        let (f, bpp) = match format {
            TexFormat::R8 => (F::R8Unorm, 1),
            TexFormat::R16 => (F::R16Unorm, 2),
            TexFormat::R16F => (F::R16Float, 2),
            TexFormat::R32F if native32 => (F::R32Float, 4),
            TexFormat::R32F => (F::Rgba16Float, 4),
            TexFormat::RG8 => (F::Rg8Unorm, 2),
            TexFormat::RG16 => (F::Rg16Unorm, 4),
            TexFormat::RG16F => (F::Rg16Float, 4),
            TexFormat::RG32F => (F::Rg32Float, 8),
            TexFormat::RGBA8 => (F::Rgba8Unorm, 4),
            TexFormat::RGBA16 => (F::Rgba16Unorm, 8),
            TexFormat::RGBA16F => (F::Rgba16Float, 8),
            TexFormat::RGBA32F if native32 => (F::Rgba32Float, 16),
            TexFormat::RGBA32F => (F::Rgba16Float, 16),
            TexFormat::Depth16 => (F::Depth16Unorm, 4),
            TexFormat::Depth24 => (F::Depth24Plus, 4),
            TexFormat::Depth32F => (F::Depth32Float, 4),
        };
        (f, bpp)
    }

    /// Does the texture of `format` keep its 32-bit floats as 16-bit ones?
    pub(super) fn f32_in_f16(&self, format: TexFormat) -> bool {
        matches!(format, TexFormat::R32F | TexFormat::RGBA32F)
            && self.storage_format(format).0 == wgpu::TextureFormat::Rgba16Float
    }

    pub(super) fn cmd_create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        layout: &Arc<ShaderLayout>,
    ) -> Result<Vec<BlockLayout>, String> {
        let pair = shader::compile_pair(&vertex_src, &fragment_src, layout)
            .inspect_err(|e| error!("wgpu: failed to create shader {id:?}: {e}"))?;
        let module =
            |label: &'static str, module: wgpu::naga::Module| self.shader_module(label, module);
        let vs = module("phx-vertex", pair.vs);
        let fs = module("phx-fragment", pair.fs);
        let groups = self.shader_group_layouts(layout, &pair.blocks);
        let bgls: Vec<Option<&wgpu::BindGroupLayout>> =
            groups.iter().map(|g| Some(&g.bgl)).collect();
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("phx-pipeline-layout"),
                bind_group_layouts: &bgls,
                immediate_size: 0,
            });
        let blocks = pair.blocks.clone();
        self.resources.insert(
            id,
            WgpuResource::Shader(Box::new(WgpuShader {
                vs,
                fs,
                fs_code: pair.fs_code,
                fs_named_outputs: pair.fs_named_outputs,
                vs_inputs: pair.vs_inputs,
                fs_outputs: pair.fs_outputs,
                groups,
                pipeline_layout,
            })),
        );
        self.bump_generation(id);
        Ok(blocks)
    }

    /// A shader module from a validated naga module of the engine's own shaders.
    ///
    /// Created without wgpu's runtime checks: the engine's shaders are trusted
    /// (they go through naga's validator here and run on GL unchecked), and
    /// the checks cost real time. Forced loop bounding in particular turns
    /// every loop into a counted one, which made the occlusion bake of
    /// generated ships several times slower than on GL.
    pub(super) fn shader_module(
        &self,
        label: &'static str,
        module: wgpu::naga::Module,
    ) -> wgpu::ShaderModule {
        let descriptor = wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        };
        #[allow(unsafe_code)]
        // SAFETY: see above; the shader sources are the engine's own.
        unsafe {
            self.device
                .create_shader_module_trusted(descriptor, wgpu::ShaderRuntimeChecks::unchecked())
        }
    }

    pub(super) fn cmd_create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-mesh-vb"),
            size: (vertices.len() as u64).next_multiple_of(4).max(4),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !vertices.is_empty() {
            let padded = pad4(&vertices);
            self.queue.write_buffer(&vertex_buffer, 0, &padded);
        }
        let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-mesh-ib"),
            size: (indices.len() as u64 * 4).max(4),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !indices.is_empty() {
            let bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
            self.queue.write_buffer(&index_buffer, 0, &bytes);
        }
        self.resources.insert(
            id,
            WgpuResource::Mesh(WgpuMesh {
                vertex_buffer,
                index_buffer,
                vertex_format,
            }),
        );
    }

    pub(super) fn cmd_destroy_resources(&mut self, ids: &[ResourceId]) {
        for id in ids {
            self.resources.remove(id);
        }
        self.views
            .retain(|key, _| !ids.iter().any(|id| id.0 == key.tex));
    }

    pub(super) fn cmd_create_pipeline(&mut self, id: PipelineId, desc: &PipelineDesc) {
        self.pipeline_descs.insert(id, desc.clone());
    }

    pub(super) fn cmd_create_sampler(&mut self, id: SamplerId, desc: &SamplerDesc) {
        self.samplers
            .insert(id, self.device.create_sampler(&sampler_descriptor(desc)));
    }

    pub(super) fn cmd_create_bind_group(
        &mut self,
        id: BindGroupId,
        group: u8,
        entries: &[BindEntry],
    ) {
        self.bind_groups.insert(
            id,
            StoredBindGroup {
                group,
                entries: entries.to_vec(),
            },
        );
    }

    pub(super) fn cmd_destroy_bind_groups(&mut self, ids: &[BindGroupId]) {
        for id in ids {
            self.bind_groups.remove(id);
        }
    }

    pub(super) fn cmd_create_buffer(&mut self, id: BufferId, size: u32) {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-arena"),
            size: size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.buffers.insert(id, buffer);
    }

    /// `WriteBuffer`: the queue orders the write before the whole next
    /// submission, so work already recorded (which may read the old bytes)
    /// is submitted first.
    pub(super) fn cmd_write_buffer(&mut self, id: BufferId, offset: u32, data: &[u8]) {
        self.flush_for_write();
        let Some(buffer) = self.buffers.get(&id) else {
            warn!("wgpu: WriteBuffer: buffer {id:?} was never created");
            return;
        };
        self.queue.write_buffer(buffer, offset as u64, &pad4(data));
    }

    // =====================================================================
    // Dispatch
    // =====================================================================

    /// Execute one command.
    pub fn execute(&mut self, cmd: RenderCommand) -> CommandReply {
        let mut reply = CommandReply::None;
        self.stats.commands_processed += 1;
        self.frame_counters.commands += 1;

        match cmd {
            RenderCommand::UpdateTexture { id, region, data } => {
                self.cmd_update_texture(id, &region, data);
            }
            RenderCommand::SetTexel1DByResource { id, x, color } => {
                self.cmd_set_texel(id, x, 0, color);
            }
            RenderCommand::SetTexel2DByResource { id, x, y, color } => {
                self.cmd_set_texel(id, x, y, color);
            }
            RenderCommand::GenerateMips { id } => self.cmd_generate_mips(id),
            RenderCommand::CopyTexture { src, dst, size } => {
                self.cmd_copy_texture(&src, &dst, size)
            }
            RenderCommand::CopyTexture2DFromFramebufferByResource { .. } => {
                self.warn_once(
                    "CopyTexture2DFromFramebuffer (Tex2D::deep_clone) is not implemented".into(),
                );
            }
            RenderCommand::ReadTextureSync {
                src,
                region,
                format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_sync(src, &region, format);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReadbackAsync {
                src,
                region,
                format,
                slot,
            } => self.cmd_readback_async(src, &region, format, slot),

            RenderCommand::BeginRenderPass(desc) => self.cmd_begin_render_pass(&desc),
            RenderCommand::EndRenderPass => self.cmd_end_render_pass(),
            RenderCommand::BeginFrame { slot } => self.cmd_begin_frame(slot),
            RenderCommand::PassCommands(mut commands) => self.cmd_pass_commands(&mut commands),
            RenderCommand::CreatePipeline { id, desc } => self.cmd_create_pipeline(id, &desc),
            RenderCommand::CreateSampler { id, desc } => self.cmd_create_sampler(id, &desc),
            RenderCommand::CreateBindGroup {
                id,
                shader: _,
                group,
                entries,
            } => self.cmd_create_bind_group(id, group, &entries),
            RenderCommand::DestroyBindGroups { ids } => self.cmd_destroy_bind_groups(&ids),
            RenderCommand::CreateBuffer { id, size } => self.cmd_create_buffer(id, size),
            RenderCommand::WriteBuffer { id, offset, data } => {
                self.cmd_write_buffer(id, offset, &data)
            }

            RenderCommand::CreateShader {
                id,
                vertex_src,
                fragment_src,
                layout,
                reply_tx,
            } => {
                let data = self.cmd_create_shader(id, vertex_src, fragment_src, &layout);
                let _ = reply_tx.send(data);
            }
            RenderCommand::CreateTexture { id, desc, data } => {
                self.cmd_create_texture(id, &desc, data);
            }
            RenderCommand::CreateMesh {
                id,
                vertices,
                indices,
                vertex_format,
            } => self.cmd_create_mesh(id, vertices, indices, vertex_format),
            RenderCommand::DestroyResources { ids } => self.cmd_destroy_resources(&ids),

            RenderCommand::Resize { width, height } => self.cmd_resize(width, height),
            RenderCommand::SetPresentMode { mode } => self.cmd_set_present_mode(mode),
            RenderCommand::SwapBuffers => {
                reply = self.cmd_swap_buffers();
            }

            RenderCommand::Flush => self.cmd_flush(),
            RenderCommand::Fence { fence_id } => {
                reply = CommandReply::Fence(fence_id);
            }
            RenderCommand::PacingFence { fence_id } => {
                reply = self.cmd_pacing_fence(fence_id);
            }
            RenderCommand::Shutdown => {}
        }
        reply
    }
}

/// `bytes` rounded up to the 4-byte multiple `write_buffer` needs.
pub(super) fn pad4(bytes: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if bytes.len() % 4 == 0 {
        std::borrow::Cow::Borrowed(bytes)
    } else {
        let mut v = bytes.to_vec();
        v.resize(bytes.len().next_multiple_of(4), 0);
        std::borrow::Cow::Owned(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{PassCommands, VertexLayout};

    /// A headless executor (no surface), or `None` without an adapter. The GPU
    /// tests are `#[ignore]`d; run them one at a time
    /// (`cargo test -p phx wgpu -- --ignored --test-threads=1`).
    fn headless() -> Option<WgpuCommandExecutor> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = crate::window::poll_startup_future(
            instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .ok()?
        .ok()?;
        let (device, queue) =
            crate::window::poll_startup_future(adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: crate::window::wanted_features(adapter.features()),
                ..Default::default()
            }))
            .ok()?
            .ok()?;
        Some(WgpuCommandExecutor::with_device(device, queue))
    }

    fn create_shader(ex: &mut WgpuCommandExecutor, id: u64, vs: &str, fs: &str) {
        let (tx, rx) = crossbeam::channel::unbounded();
        ex.execute(RenderCommand::CreateShader {
            id: ResourceId(id),
            vertex_src: vs.to_string(),
            fragment_src: fs.to_string(),
            layout: Arc::new(ShaderLayout::default()),
            reply_tx: tx,
        });
        rx.recv().unwrap().expect("shader compiles");
    }

    fn read(ex: &mut WgpuCommandExecutor, id: u64, desc: &TexDesc) -> Vec<u8> {
        let (tx, rx) = crossbeam::channel::unbounded();
        ex.execute(RenderCommand::ReadTextureSync {
            src: crate::render::ReadSource::Texture(ResourceId(id)),
            region: TexRegion::level(desc, 0),
            format: TexFormat::RGBA8,
            reply_tx: tx,
        });
        rx.recv().unwrap()
    }

    /// The convention of the module docs: a fullscreen pass writing its uv
    /// puts uv.y = 0 in row 0, as GL does.
    #[test]
    #[ignore = "needs a GPU adapter"]
    fn fullscreen_uv_lands_in_gl_orientation() {
        let Some(mut ex) = headless() else {
            eprintln!("no adapter, skipping");
            return;
        };
        create_shader(
            &mut ex,
            1,
            "#version 330\nin vec3 vertex_position;\nin vec2 vertex_uv;\nout vec2 uv;\nvoid main() { uv = vertex_uv; gl_Position = vec4(2.0 * vertex_position.xy - 1.0, 0.0, 1.0); }\n",
            "#version 330\nin vec2 uv;\nlayout(location = 0) out vec4 outColor;\nvoid main() { outColor = vec4(uv.x, uv.y, 0.0, 1.0); }\n",
        );
        let desc = TexDesc::d2(4, 4, TexFormat::RGBA8);
        ex.execute(RenderCommand::CreateTexture {
            id: ResourceId(2),
            desc: Box::new(desc),
            data: None,
        });
        let mut pipeline = PipelineDesc::new(ResourceId(1));
        pipeline.vertex = VertexLayout::Fullscreen;
        ex.execute(RenderCommand::CreatePipeline {
            id: PipelineId(0),
            desc: Box::new(pipeline),
        });
        let view = TexView::new(ResourceId(2), ViewDim::D2, 0, [4, 4]);
        ex.execute(RenderCommand::BeginRenderPass(Box::new(
            RenderPassDesc::with_color("test", view, LoadOp::Clear, [0.0; 4]),
        )));
        ex.execute(RenderCommand::PassCommands(Box::new(PassCommands {
            slot: 0,
            uniforms: Vec::new(),
            vertices: Vec::new(),
            cmds: vec![PassCmd::SetPipeline(PipelineId(0)), PassCmd::DrawFullscreen],
        })));
        ex.execute(RenderCommand::EndRenderPass);

        let texels = read(&mut ex, 2, &desc);
        assert_eq!(texels.len(), 4 * 4 * 4);
        let at = |x: usize, row: usize| {
            let i = (row * 4 + x) * 4;
            (texels[i], texels[i + 1])
        };
        // Texel centres: u = (x + 0.5) / 4, v = (row + 0.5) / 4, row 0 first.
        let near = |a: u8, b: f32| (a as f32 - b * 255.0).abs() <= 1.5;
        assert!(
            near(at(0, 0).0, 0.125) && near(at(0, 0).1, 0.125),
            "{:?}",
            at(0, 0)
        );
        assert!(
            near(at(3, 0).0, 0.875) && near(at(3, 0).1, 0.125),
            "{:?}",
            at(3, 0)
        );
        assert!(
            near(at(0, 3).0, 0.125) && near(at(0, 3).1, 0.875),
            "{:?}",
            at(0, 3)
        );
    }

    /// Depth: a nearer quad hides a farther one whatever the draw order, with
    /// the clip-space z remap and the depth test of the pipeline.
    #[test]
    #[ignore = "needs a GPU adapter"]
    fn depth_test_keeps_the_nearer_quad() {
        let Some(mut ex) = headless() else {
            eprintln!("no adapter, skipping");
            return;
        };
        // gl_FragDepth overrides the rasterized depth, as the engine's logarithmic depth does.
        create_shader(
            &mut ex,
            1,
            "#version 330\nin vec3 vertex_position;\nout float shade;\nvoid main() { shade = vertex_position.x; gl_Position = vec4(2.0 * vertex_position.xy - 1.0, 0.0, 1.0); }\n",
            "#version 330\nin float shade;\nlayout(location = 0) out vec4 outColor;\nvoid main() { outColor = vec4(1.0, 1.0, 1.0, 1.0); gl_FragDepth = 0.5; }\n",
        );
        let desc = TexDesc::d2(2, 2, TexFormat::RGBA8);
        for (id, d) in [(2u64, desc), (3, TexDesc::d2(2, 2, TexFormat::Depth32F))] {
            ex.execute(RenderCommand::CreateTexture {
                id: ResourceId(id),
                desc: Box::new(d),
                data: None,
            });
        }
        let mut pass = RenderPassDesc::with_color(
            "depth",
            TexView::new(ResourceId(2), ViewDim::D2, 0, [2, 2]),
            LoadOp::Clear,
            [0.0; 4],
        );
        pass.set_depth(
            TexView::new(ResourceId(3), ViewDim::D2, 0, [2, 2]),
            LoadOp::Clear,
            1.0,
        );
        let mut pipeline = PipelineDesc::new(ResourceId(1));
        pipeline.vertex = VertexLayout::Fullscreen;
        pipeline.depth = crate::render::DepthState {
            test: true,
            write: true,
            compare: CompareFn::LessEqual,
        };
        ex.execute(RenderCommand::CreatePipeline {
            id: PipelineId(0),
            desc: Box::new(pipeline),
        });
        ex.execute(RenderCommand::BeginRenderPass(Box::new(pass)));
        ex.execute(RenderCommand::PassCommands(Box::new(PassCommands {
            slot: 0,
            uniforms: Vec::new(),
            vertices: Vec::new(),
            cmds: vec![PassCmd::SetPipeline(PipelineId(0)), PassCmd::DrawFullscreen],
        })));
        ex.execute(RenderCommand::EndRenderPass);
        // gl_FragDepth 0.5 passes against the cleared 1.0: the quad is drawn.
        let texels = read(&mut ex, 2, &desc);
        assert_eq!(&texels[..4], &[255, 255, 255, 255]);
    }
}
