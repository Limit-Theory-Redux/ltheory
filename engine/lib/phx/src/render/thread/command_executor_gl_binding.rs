//! GL 3.3 implementation of the binding model (doc/engine/render-api-v2.md,
//! section 4): pipelines with a diffed state cache, sampler objects, bind
//! groups, the uniform ring buffers, pass command execution, and the link-time
//! application and reflection of a shader's `#group` layout.

#![allow(unsafe_code)]

use std::collections::HashMap;

use tracing::{debug, error, warn};

use super::{AttachKey, GpuResource, MAX_TEXTURE_SLOTS, TextureBinding, TextureType};
use crate::render::{
    BindEntry, BindGroupId, BlendMode, BlockLayout, BlockMember, BufferId, CHUNK_SIZE,
    CommandCategory, CommandExecutor, CompareFn, CullFace, GROUP_COUNT, GROUP_DRAW, GROUP_FRAME,
    GROUP_INPUTS, GlslType, ImmLayout, InstanceData, MAX_FRAMES_IN_FLIGHT, MAX_INPUTS, PassCmd,
    PassCommands, PipelineDesc, PipelineId, PolygonMode, ResourceId, ReturnedChunk, RingChunk,
    SamplerDesc, SamplerId, Samplers, ShaderLayout, TexDesc, TexView, UNIFORM_ALIGN,
    VERTEX_CHUNK_SIZE, ViewDim, block_binding, entry_unit, gl, texture_unit,
};

/// Uniform block binding points (`group * 4 + k`) the passes drive.
const VIEW_BINDING: u32 = block_binding(GROUP_FRAME, 0);
const DRAW_BINDING: u32 = block_binding(GROUP_DRAW, 0);
const UBO_BINDINGS: usize = 16;
/// Vertex attribute locations of the shape parameters of `Imm2DVertex`
/// (`imm_params`, `imm_params2` in `vertex/imm2d.glsl`).
pub(super) const IMM_PARAMS_LOCATION: u32 = 11;
pub(super) const IMM_PARAMS2_LOCATION: u32 = 12;
/// Texture units of the group-0 environment maps (`envMap`, `irMap`).
const ENV_UNIT: u32 = texture_unit(GROUP_FRAME, 0);
const IR_UNIT: u32 = texture_unit(GROUP_FRAME, 1);
/// Pass inputs start at the first unit of group 3.
const INPUT_UNIT: u32 = texture_unit(GROUP_INPUTS, 0);

/// What the executor believes GL's fixed-function state is. `None` = unknown.
/// Pipelines diff against it.
#[derive(Debug, Default)]
pub(super) struct GlStateCache {
    pub blend: Option<BlendMode>,
    pub cull: Option<CullFace>,
    pub depth_test: Option<bool>,
    pub depth_write: Option<bool>,
    pub depth_func: Option<CompareFn>,
    pub polygon: Option<PolygonMode>,
}

/// One uniform buffer range bound to a binding point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct UboBinding {
    buffer: u32,
    offset: isize,
    size: isize,
}

pub(super) struct GlBindGroup {
    group: u8,
    entries: Vec<BindEntry>,
}

/// Executor-side state of the binding model.
pub(super) struct GlBindingState {
    pub pipelines: HashMap<PipelineId, PipelineDesc>,
    pub samplers: HashMap<SamplerId, u32>,
    pub bind_groups: HashMap<BindGroupId, GlBindGroup>,
    /// GL uniform buffers by `BufferId` (material parameter arenas).
    pub buffers: HashMap<BufferId, u32>,
    pub gl_state: GlStateCache,
    /// The pipeline whose program and state are applied.
    pub current_pipeline: Option<PipelineId>,
    /// Topology of the current pipeline's draws.
    pub topology: u32,
    /// GL uniform buffers of the uniform ring, per frame slot and chunk index.
    pub ring_buffers: Vec<Vec<u32>>,
    /// GL array buffers of the vertex ring, per frame slot and chunk index.
    pub vertex_buffers: Vec<Vec<u32>>,
    /// Frame slot the executor is in (`BeginFrame`).
    pub frame_slot: usize,
    /// Per slot: the fence inserted after the slot's frame, a `GLsync` as an
    /// integer (0 = none). Waited on before the slot is reused.
    pub slot_fences: [usize; MAX_FRAMES_IN_FLIGHT],
    /// Ring memory uploaded and ready to go back to the main thread.
    pub returned: Vec<ReturnedChunk>,
    pub ubo_bound: [UboBinding; UBO_BINDINGS],
    /// Sampler object bound to each texture unit (0 = none).
    pub unit_samplers: [u32; MAX_TEXTURE_SLOTS],
    /// Mip range last set on each texture (`TEXTURE_BASE_LEVEL`, `MAX_LEVEL`).
    pub mip_ranges: HashMap<ResourceId, (i32, i32)>,
    /// What every live texture was created as (an update needs its format and kind).
    pub tex_descs: HashMap<ResourceId, TexDesc>,
    /// Unit quad for `DrawFullscreen`.
    pub fullscreen_vao: u32,
    pub fullscreen_vbo: u32,
    /// Vertex arrays of the immediate batcher (`DrawImm`): attribute
    /// pointers are set per draw at the ring offset.
    pub imm2d_vao: u32,
    pub imm3d_vao: u32,
    /// Scratch read and draw framebuffers of `CopyTexture` (0 until first use).
    pub copy_fbos: [u32; 2],
}

impl GlBindingState {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            samplers: HashMap::new(),
            bind_groups: HashMap::new(),
            buffers: HashMap::new(),
            gl_state: GlStateCache::default(),
            current_pipeline: None,
            topology: gl::TRIANGLES,
            ring_buffers: (0..MAX_FRAMES_IN_FLIGHT).map(|_| Vec::new()).collect(),
            vertex_buffers: (0..MAX_FRAMES_IN_FLIGHT).map(|_| Vec::new()).collect(),
            frame_slot: 0,
            slot_fences: [0; MAX_FRAMES_IN_FLIGHT],
            returned: Vec::new(),
            ubo_bound: [UboBinding::default(); UBO_BINDINGS],
            unit_samplers: [0; MAX_TEXTURE_SLOTS],
            mip_ranges: HashMap::new(),
            tex_descs: HashMap::new(),
            fullscreen_vao: 0,
            fullscreen_vbo: 0,
            imm2d_vao: 0,
            imm3d_vao: 0,
            copy_fbos: [0; 2],
        }
    }
}

/// `LTHEORY_GL_CHECK=1`: log every GL error right after the pass command that
/// raised it (a glGetError per command, so off by default).
fn gl_check_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("LTHEORY_GL_CHECK").is_some())
}

fn check_gl(what: &dyn std::fmt::Debug) {
    if !gl_check_enabled() {
        return;
    }
    loop {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            break;
        }
        error!("GL error {} after {what:?}", gl::error_to_str(err));
    }
}

impl CommandExecutor {
    // -----------------------------------------------------------------
    // Setup
    // -----------------------------------------------------------------

    /// The unit quad `DrawFullscreen` draws, laid out like an `ImmVertex`
    /// (same attribute locations as every mesh): positions are `[0,1]^2`,
    /// the vertex shader scales them to the viewport. Same winding and
    /// texture coordinates as `Draw.Rect`, drawn as the same triangle fan.
    pub(super) fn init_fullscreen_quad(&mut self) {
        use crate::render::ImmVertex;
        let vertex = |x: f32, y: f32| ImmVertex {
            pos: [x, y, 0.0],
            normal: [0.0, 0.0, 0.0],
            uv: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let vertices = [
            vertex(0.0, 0.0),
            vertex(0.0, 1.0),
            vertex(1.0, 1.0),
            vertex(1.0, 0.0),
        ];
        let stride = std::mem::size_of::<ImmVertex>() as i32;
        unsafe {
            let b = &mut self.binding;
            gl::GenVertexArrays(1, &mut b.fullscreen_vao);
            gl::GenBuffers(1, &mut b.fullscreen_vbo);
            gl::BindVertexArray(b.fullscreen_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, b.fullscreen_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&vertices) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            for (location, components, offset) in
                [(0u32, 3, 0usize), (1, 3, 12), (2, 2, 24), (3, 4, 32)]
            {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(
                    location,
                    components,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    offset as *const _,
                );
            }
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // Immediate batcher layouts: which attributes are enabled is
            // fixed per VAO; the pointers follow the ring buffer per draw.
            // 2D: 0 position, 2 uv, 3 color, 11 and 12 shape parameters.
            gl::GenVertexArrays(1, &mut b.imm2d_vao);
            gl::BindVertexArray(b.imm2d_vao);
            for location in [0u32, 2, 3, IMM_PARAMS_LOCATION, IMM_PARAMS2_LOCATION] {
                gl::EnableVertexAttribArray(location);
            }
            // 3D: 0 position, 2 uv, 3 color.
            gl::GenVertexArrays(1, &mut b.imm3d_vao);
            gl::BindVertexArray(b.imm3d_vao);
            for location in [0u32, 2, 3] {
                gl::EnableVertexAttribArray(location);
            }
            gl::BindVertexArray(0);
        }
    }

    /// Attach `view` (a level of a 2D texture, a cube face or a 3D slice) to
    /// color attachment 0 of the framebuffer bound to `target`. `false` if
    /// the texture is missing or the view is not attachable.
    fn attach_copy_view(&self, target: u32, view: &TexView) -> bool {
        let level = view.base_mip as i32;
        unsafe {
            match (self.resources.get(&view.tex), view.dim) {
                (Some(GpuResource::Texture2D { handle }), ViewDim::D2) => {
                    gl::FramebufferTexture2D(
                        target,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D,
                        *handle,
                        level,
                    );
                }
                (Some(GpuResource::TextureCube { handle }), ViewDim::CubeFace(face)) => {
                    gl::FramebufferTexture2D(
                        target,
                        gl::COLOR_ATTACHMENT0,
                        face as u32,
                        *handle,
                        level,
                    );
                }
                (Some(GpuResource::Texture3D { handle }), ViewDim::D2Layer(layer)) => {
                    gl::FramebufferTextureLayer(
                        target,
                        gl::COLOR_ATTACHMENT0,
                        *handle,
                        level,
                        layer as i32,
                    );
                }
                (resource, dim) => {
                    error!(
                        "CopyTexture: {:?} ({dim:?}) is not a copyable view (resource: {})",
                        view.tex,
                        if resource.is_some() {
                            "wrong texture kind"
                        } else {
                            "not found"
                        }
                    );
                    return false;
                }
            }
        }
        true
    }

    /// `CopyTexture`: `glBlitFramebuffer` from `src` to `dst` through two
    /// scratch framebuffers (`glCopyImageSubData` needs GL 4.3). Same-format
    /// copies are exact: the blit is nearest-filtered and unscaled. The pass
    /// framebuffer, scissor and draw-buffer state are restored.
    pub(super) fn cmd_copy_texture(&mut self, src: &TexView, dst: &TexView, size: [u32; 3]) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        let (w, h) = (size[0] as i32, size[1] as i32);
        if w <= 0 || h <= 0 {
            return;
        }
        unsafe {
            if self.binding.copy_fbos[0] == 0 {
                gl::GenFramebuffers(2, self.binding.copy_fbos.as_mut_ptr());
            }
            let [read_fbo, draw_fbo] = self.binding.copy_fbos;
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, read_fbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, draw_fbo);
            let ok = self.attach_copy_view(gl::READ_FRAMEBUFFER, src)
                && self.attach_copy_view(gl::DRAW_FRAMEBUFFER, dst);
            if ok {
                gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
                let draw_buffers = [gl::COLOR_ATTACHMENT0];
                gl::DrawBuffers(1, draw_buffers.as_ptr());
                let scissor = gl::IsEnabled(gl::SCISSOR_TEST) == gl::TRUE;
                if scissor {
                    gl::Disable(gl::SCISSOR_TEST);
                }
                gl::BlitFramebuffer(0, 0, w, h, 0, 0, w, h, gl::COLOR_BUFFER_BIT, gl::NEAREST);
                if scissor {
                    gl::Enable(gl::SCISSOR_TEST);
                }
            }
            // Detach, so a deleted texture is not kept alive by the scratch FBOs.
            for (target, fbo) in [
                (gl::READ_FRAMEBUFFER, read_fbo),
                (gl::DRAW_FRAMEBUFFER, draw_fbo),
            ] {
                gl::BindFramebuffer(target, fbo);
                gl::FramebufferTexture2D(target, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, 0, 0);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.bound_fbo);
        }
    }

    // -----------------------------------------------------------------
    // Object creation
    // -----------------------------------------------------------------

    pub(super) fn cmd_create_pipeline(&mut self, id: PipelineId, desc: &PipelineDesc) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        self.binding.pipelines.insert(id, desc.clone());
    }

    pub(super) fn cmd_create_sampler(&mut self, id: SamplerId, desc: &SamplerDesc) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        if !self.has_gl_context() {
            return;
        }
        let mut sampler = 0;
        unsafe {
            gl::GenSamplers(1, &mut sampler);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER, desc.gl_min_filter() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, desc.gl_mag_filter() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_S, desc.wrap[0] as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_T, desc.wrap[1] as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_R, desc.wrap[2] as i32);
            if desc.anisotropy > 1 {
                gl::SamplerParameterf(
                    sampler,
                    gl::TEXTURE_MAX_ANISOTROPY_EXT,
                    desc.anisotropy as f32,
                );
            }
            gl::SamplerParameterf(sampler, gl::TEXTURE_MIN_LOD, desc.lod_min as f32);
            gl::SamplerParameterf(sampler, gl::TEXTURE_MAX_LOD, desc.lod_max as f32);
            if let Some(compare) = desc.compare {
                gl::SamplerParameteri(
                    sampler,
                    gl::TEXTURE_COMPARE_MODE,
                    gl::COMPARE_REF_TO_TEXTURE as i32,
                );
                gl::SamplerParameteri(sampler, gl::TEXTURE_COMPARE_FUNC, compare.to_gl() as i32);
            }
        }
        self.binding.samplers.insert(id, sampler);
    }

    pub(super) fn cmd_create_bind_group(
        &mut self,
        id: BindGroupId,
        _shader: ResourceId,
        group: u8,
        entries: &[BindEntry],
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        self.binding.bind_groups.insert(
            id,
            GlBindGroup {
                group,
                entries: entries.to_vec(),
            },
        );
    }

    pub(super) fn cmd_destroy_bind_groups(&mut self, ids: &[BindGroupId]) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        for id in ids {
            self.binding.bind_groups.remove(id);
        }
    }

    pub(super) fn cmd_create_buffer(&mut self, id: BufferId, size: u32) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        if !self.has_gl_context() {
            return;
        }
        let mut buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::UNIFORM_BUFFER, buffer);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
        self.binding.buffers.insert(id, buffer);
    }

    pub(super) fn cmd_write_buffer(&mut self, id: BufferId, offset: u32, data: &[u8]) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let Some(&buffer) = self.binding.buffers.get(&id) else {
            if self.has_gl_context() {
                warn!("WriteBuffer: buffer {id:?} was never created");
            }
            return;
        };
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, buffer);
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                offset as isize,
                data.len() as isize,
                data.as_ptr() as *const _,
            );
        }
    }

    // -----------------------------------------------------------------
    // Pipelines
    // -----------------------------------------------------------------

    /// Make `id` the current pipeline: bind its program and diff its state
    /// against the cache.
    fn apply_pipeline(&mut self, id: PipelineId) {
        if self.binding.current_pipeline == Some(id) {
            return;
        }
        let Some(desc) = self.binding.pipelines.get(&id) else {
            error!("SetPipeline: pipeline {id:?} was never created");
            return;
        };
        let program = match self.resources.get(&desc.shader) {
            Some(GpuResource::Shader { program }) => *program,
            _ => {
                warn!("SetPipeline: shader {:?} not found", desc.shader);
                return;
            }
        };
        let (blend, cull, depth, polygon, topology) = (
            desc.blend,
            desc.cull,
            desc.depth,
            desc.polygon,
            desc.topology.primitive().to_gl(),
        );

        if self.current_program != program {
            self.this_frame_stats.shader_distinct_programs += 1;
            unsafe {
                gl::UseProgram(program);
            }
            self.current_program = program;
        } else {
            self.this_frame_stats.shader_redundant_binds += 1;
        }
        self.this_frame_stats.shader_bind_commands += 1;

        let state = &mut self.binding.gl_state;
        unsafe {
            if state.blend != Some(blend) {
                apply_blend(blend);
                state.blend = Some(blend);
            }
            if state.cull != Some(cull) {
                apply_cull(cull);
                state.cull = Some(cull);
            }
            if state.depth_test != Some(depth.test) {
                if depth.test {
                    gl::Enable(gl::DEPTH_TEST);
                } else {
                    gl::Disable(gl::DEPTH_TEST);
                }
                state.depth_test = Some(depth.test);
            }
            if state.depth_write != Some(depth.write) {
                gl::DepthMask(if depth.write { gl::TRUE } else { gl::FALSE });
                state.depth_write = Some(depth.write);
            }
            if state.depth_func != Some(depth.compare) {
                gl::DepthFunc(depth.compare.to_gl());
                state.depth_func = Some(depth.compare);
            }
            if state.polygon != Some(polygon) {
                gl::PolygonMode(
                    gl::FRONT_AND_BACK,
                    if polygon == PolygonMode::Line {
                        gl::LINE
                    } else {
                        gl::FILL
                    },
                );
                state.polygon = Some(polygon);
            }
        }
        self.binding.topology = topology;
        self.binding.current_pipeline = Some(id);
    }

    /// A pass begins: fixed-function state returns to the defaults (no blend,
    /// no culling, no depth test, depth writes on, `LEQUAL`, filled polygons),
    /// so draws that do not set a pipeline see a known state and nothing leaks
    /// from the previous pass's last pipeline. wgpu passes start from scratch
    /// too.
    pub(super) fn reset_pass_state(&mut self) {
        let state = &mut self.binding.gl_state;
        unsafe {
            if state.blend != Some(BlendMode::Disabled) {
                apply_blend(BlendMode::Disabled);
                state.blend = Some(BlendMode::Disabled);
            }
            if state.cull != Some(CullFace::None) {
                apply_cull(CullFace::None);
                state.cull = Some(CullFace::None);
            }
            if state.depth_test != Some(false) {
                gl::Disable(gl::DEPTH_TEST);
                state.depth_test = Some(false);
            }
            if state.depth_write != Some(true) {
                gl::DepthMask(gl::TRUE);
                state.depth_write = Some(true);
            }
            if state.depth_func != Some(CompareFn::LessEqual) {
                gl::DepthFunc(gl::LEQUAL);
                state.depth_func = Some(CompareFn::LessEqual);
            }
            if state.polygon != Some(PolygonMode::Fill) {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                state.polygon = Some(PolygonMode::Fill);
            }
        }
        self.binding.current_pipeline = None;
    }

    // -----------------------------------------------------------------
    // Uniform ring
    // -----------------------------------------------------------------

    /// GL buffer of ring chunk `index` in `slot`, created on first use.
    /// Reuse across frames is safe because `BeginFrame` waits for the slot's
    /// fence before the slot's chunks are overwritten.
    fn ring_buffer(&mut self, slot: usize, index: usize, vertex: bool) -> u32 {
        let (buffers, target, size) = if vertex {
            (
                &mut self.binding.vertex_buffers[slot],
                gl::ARRAY_BUFFER,
                VERTEX_CHUNK_SIZE,
            )
        } else {
            (
                &mut self.binding.ring_buffers[slot],
                gl::UNIFORM_BUFFER,
                CHUNK_SIZE,
            )
        };
        while buffers.len() <= index {
            let mut buffer = 0;
            unsafe {
                gl::GenBuffers(1, &mut buffer);
                gl::BindBuffer(target, buffer);
                gl::BufferData(target, size as isize, std::ptr::null(), gl::STREAM_DRAW);
            }
            buffers.push(buffer);
        }
        buffers[index]
    }

    /// Upload ring runs into the slot's buffers, then queue their memory for
    /// return to the main thread.
    fn upload_ring_chunks(&mut self, slot: usize, chunks: &mut [RingChunk], vertex: bool) {
        let target = if vertex {
            gl::ARRAY_BUFFER
        } else {
            gl::UNIFORM_BUFFER
        };
        for chunk in chunks.iter_mut() {
            let buffer = self.ring_buffer(slot, chunk.at.buffer as usize, vertex);
            let data = chunk.data();
            unsafe {
                gl::BindBuffer(target, buffer);
                gl::BufferSubData(
                    target,
                    chunk.at.offset as isize,
                    data.len() as isize,
                    data.as_ptr() as *const _,
                );
            }
            self.binding.returned.push(ReturnedChunk {
                vertex,
                bytes: std::mem::take(&mut chunk.bytes),
            });
        }
        if vertex && !chunks.is_empty() {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
    }

    /// Hand back the ring memory uploaded since the last call.
    pub fn take_returned_chunks(&mut self) -> Vec<ReturnedChunk> {
        std::mem::take(&mut self.binding.returned)
    }

    // -----------------------------------------------------------------
    // Frame slots and fences
    // -----------------------------------------------------------------

    /// A new frame starts in ring slot `slot`: wait until the GPU has
    /// finished the frame that last used it (`glClientWaitSync` on the fence
    /// `SwapBuffers` inserted after that frame's last pass), then reuse the
    /// slot's ring buffers.
    pub(super) fn cmd_begin_frame(&mut self, slot: u8) {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        let slot = slot as usize % MAX_FRAMES_IN_FLIGHT;
        self.binding.frame_slot = slot;
        if !self.has_gl_context() {
            return;
        }
        // Readbacks that finished since the last frame (never waits).
        self.poll_readbacks();
        let fence = std::mem::take(&mut self.binding.slot_fences[slot]);
        if fence == 0 {
            return;
        }
        unsafe {
            let sync = fence as gl::types::GLsync;
            let mut status = gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            if status == gl::TIMEOUT_EXPIRED {
                // The GPU is behind. Wait in slices so a lost context cannot
                // hang the render thread forever.
                let started = std::time::Instant::now();
                while status == gl::TIMEOUT_EXPIRED
                    && started.elapsed() < std::time::Duration::from_secs(5)
                {
                    status = gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000);
                }
                if status == gl::TIMEOUT_EXPIRED {
                    error!("BeginFrame: slot {slot} fence did not signal within 5 s");
                }
            }
            gl::DeleteSync(sync);
        }
    }

    /// Frame end: fence the commands the slot's frame issued. Called by
    /// `SwapBuffers` just before the swap.
    pub(super) fn insert_slot_fence(&mut self) {
        if !self.has_gl_context() {
            return;
        }
        let slot = self.binding.frame_slot;
        unsafe {
            let old = std::mem::take(&mut self.binding.slot_fences[slot]);
            if old != 0 {
                gl::DeleteSync(old as gl::types::GLsync);
            }
            let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            self.binding.slot_fences[slot] = sync as usize;
        }
    }

    fn bind_ubo_range(&mut self, binding: u32, buffer: u32, offset: u32, size: u32) {
        debug_assert!(offset % UNIFORM_ALIGN == 0);
        let want = UboBinding {
            buffer,
            offset: offset as isize,
            size: size as isize,
        };
        let cached = &mut self.binding.ubo_bound[binding as usize];
        if *cached == want {
            return;
        }
        *cached = want;
        unsafe {
            gl::BindBufferRange(gl::UNIFORM_BUFFER, binding, buffer, want.offset, want.size);
        }
    }

    // -----------------------------------------------------------------
    // Texture binding
    // -----------------------------------------------------------------

    /// Bind `view` to `unit` with `sampler`. Goes through the same per-unit
    /// texture cache as the legacy path, plus per-unit sampler and
    /// per-texture mip-range caches.
    fn bind_view(&mut self, unit: u32, view: &TexView, sampler: SamplerId) {
        let (target, handle, ty) = match self.resources.get(&view.tex) {
            Some(GpuResource::Texture1D { handle }) => {
                (gl::TEXTURE_1D, *handle, TextureType::Texture1D)
            }
            Some(GpuResource::Texture2D { handle }) => {
                (gl::TEXTURE_2D, *handle, TextureType::Texture2D)
            }
            Some(GpuResource::Texture3D { handle }) => {
                (gl::TEXTURE_3D, *handle, TextureType::Texture3D)
            }
            Some(GpuResource::TextureCube { handle }) => {
                (gl::TEXTURE_CUBE_MAP, *handle, TextureType::TextureCube)
            }
            _ => {
                warn!("bind view: texture {:?} not found", view.tex);
                return;
            }
        };
        if !matches!(
            view.dim,
            ViewDim::D1 | ViewDim::D2 | ViewDim::D3 | ViewDim::Cube
        ) {
            debug!(
                "bind view: {:?} of {:?} samples the whole texture on GL 3.3",
                view.dim, view.tex
            );
        }
        let unit_index = unit as usize;
        debug_assert!(unit_index < MAX_TEXTURE_SLOTS);

        // Mip range ("view" of levels): only touch the texture when it
        // differs from what it already has.
        let want_range = view.gl_mip_range();
        let have_range = self
            .binding
            .mip_ranges
            .get(&view.tex)
            .copied()
            .unwrap_or((0, 1000));
        let range_differs = have_range != want_range;
        let needs_bind = {
            let cur = &self.texture_bindings[unit_index];
            !(cur.handle == handle && cur.tex_type == Some(ty))
        };

        if needs_bind || range_differs {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                if needs_bind {
                    gl::BindTexture(target, handle);
                    self.this_frame_stats.texture_bind_calls += 1;
                }
                if range_differs {
                    // The texture may not be bound if only the range
                    // differs; bind it on this unit (a no-op rebind when
                    // it is) before setting parameters.
                    if !needs_bind {
                        gl::BindTexture(target, handle);
                    }
                    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, want_range.0);
                    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, want_range.1);
                }
                gl::ActiveTexture(gl::TEXTURE0);
            }
            if needs_bind {
                self.texture_bindings[unit_index] = TextureBinding::new(handle, ty);
            }
            if range_differs {
                self.binding.mip_ranges.insert(view.tex, want_range);
            }
        } else {
            self.texture_binds_skipped += 1;
            self.this_frame_stats.texture_binds_skipped += 1;
        }

        let sampler_object = self.binding.samplers.get(&sampler).copied().unwrap_or(0);
        if self.binding.unit_samplers[unit_index] != sampler_object {
            unsafe {
                gl::BindSampler(unit, sampler_object);
            }
            self.binding.unit_samplers[unit_index] = sampler_object;
        }
    }

    /// Unbind whatever cube map is on `unit` (the environment slots).
    fn unbind_cube(&mut self, unit: u32) {
        let unit_index = unit as usize;
        let cur = self.texture_bindings[unit_index];
        if cur.handle == 0 {
            return;
        }
        if let Some(ty) = cur.tex_type {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(ty.to_gl_target(), 0);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            self.texture_bindings[unit_index] = TextureBinding::unbound();
        }
    }

    // -----------------------------------------------------------------
    // Pass commands
    // -----------------------------------------------------------------

    pub(super) fn cmd_pass_commands(&mut self, commands: &mut PassCommands) {
        let _sa = self.record_command(CommandCategory::Draw, false, false);
        if !self.has_gl_context() {
            // Nothing to upload to, but the memory still goes back.
            for (chunks, vertex) in [
                (&mut commands.uniforms, false),
                (&mut commands.vertices, true),
            ] {
                for chunk in chunks.iter_mut() {
                    self.binding.returned.push(ReturnedChunk {
                        vertex,
                        bytes: std::mem::take(&mut chunk.bytes),
                    });
                }
            }
            return;
        }
        let slot = commands.slot as usize % MAX_FRAMES_IN_FLIGHT;
        check_gl(&"commands before this PassCommands");

        // Uploads first: everything the commands reference is in these.
        self.upload_ring_chunks(slot, &mut commands.uniforms, false);
        self.upload_ring_chunks(slot, &mut commands.vertices, true);

        check_gl(&"ring upload");
        for cmd in &commands.cmds {
            self.run_pass_cmd(cmd, slot);
            check_gl(cmd);
        }
    }

    fn run_pass_cmd(&mut self, cmd: &PassCmd, slot: usize) {
        {
            match cmd {
                PassCmd::SetPipeline(id) => {
                    self.apply_pipeline(*id);
                    #[cfg(feature = "stats-server")]
                    self.count_state_change();
                }
                PassCmd::SetBindGroup { group, id } => {
                    self.this_frame_stats.bind_group_switches += 1;
                    let Some(bg) = self.binding.bind_groups.get(id) else {
                        error!("SetBindGroup: bind group {id:?} was never created");
                        return;
                    };
                    let entries = bg.entries.clone();
                    let group = *group;
                    debug_assert_eq!(group, bg.group);
                    for entry in &entries {
                        match entry {
                            BindEntry::Texture { view, sampler, .. } => {
                                if let Some(unit) = entry_unit(group, entry) {
                                    self.bind_view(unit, view, *sampler);
                                }
                            }
                            BindEntry::Uniform {
                                index,
                                buffer,
                                offset,
                                size,
                            } => {
                                let Some(&gl_buffer) = self.binding.buffers.get(buffer) else {
                                    warn!("SetBindGroup: buffer {buffer:?} was never created");
                                    continue;
                                };
                                self.bind_ubo_range(
                                    block_binding(group, *index),
                                    gl_buffer,
                                    *offset,
                                    *size,
                                );
                            }
                        }
                    }
                }
                PassCmd::SetView { block, size } => {
                    let buffer = self.ring_buffer(slot, block.buffer as usize, false);
                    self.bind_ubo_range(VIEW_BINDING, buffer, block.offset, *size);
                }
                PassCmd::SetEnvironment { env_map, ir_map } => {
                    let sampler = Samplers::LinearMipClamp.id();
                    for (unit, id) in [(ENV_UNIT, env_map), (IR_UNIT, ir_map)] {
                        match id {
                            Some(id) => {
                                let view = TexView::full(*id, ViewDim::Cube, [1, 1]);
                                self.bind_view(unit, &view, sampler);
                            }
                            None => self.unbind_cube(unit),
                        }
                    }
                }
                PassCmd::SetDraw { at, size } => {
                    let buffer = self.ring_buffer(slot, at.buffer as usize, false);
                    self.bind_ubo_range(DRAW_BINDING, buffer, at.offset, *size);
                }
                PassCmd::SetInputs(inputs) => {
                    for (i, input) in inputs.iter().enumerate().take(MAX_INPUTS) {
                        if let Some((view, sampler)) = input {
                            self.bind_view(INPUT_UNIT + i as u32, view, *sampler);
                        }
                    }
                }
                PassCmd::SetViewport([x, y, w, h]) => unsafe {
                    gl::Viewport(*x, *y, *w, *h);
                },
                PassCmd::SetScissor(rect) => unsafe {
                    match rect {
                        Some([x, y, w, h]) => {
                            gl::Enable(gl::SCISSOR_TEST);
                            gl::Scissor(*x, *y, *w, *h);
                        }
                        None => gl::Disable(gl::SCISSOR_TEST),
                    }
                },
                PassCmd::DrawMesh {
                    mesh,
                    first_index,
                    index_count,
                } => {
                    let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(mesh) else {
                        warn!("DrawMesh: mesh {mesh:?} not found");
                        return;
                    };
                    unsafe {
                        gl::BindVertexArray(*vao);
                        gl::DrawElements(
                            self.binding.topology,
                            *index_count as i32,
                            gl::UNSIGNED_INT,
                            (*first_index as usize * 4) as *const _,
                        );
                        gl::BindVertexArray(0);
                    }
                    self.this_frame_stats.draw_mesh_calls += 1;
                    self.this_frame_stats.vertices_drawn += *index_count as u64;
                    #[cfg(feature = "stats-server")]
                    self.count_draw();
                }
                PassCmd::DrawFullscreen => {
                    unsafe {
                        gl::BindVertexArray(self.binding.fullscreen_vao);
                        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
                        gl::BindVertexArray(0);
                    }
                    self.this_frame_stats.draw_immediate_calls += 1;
                    self.this_frame_stats.vertices_drawn += 4;
                    #[cfg(feature = "stats-server")]
                    self.count_draw();
                }
                PassCmd::DrawImm {
                    layout,
                    vertices,
                    count,
                } => {
                    let buffer = self.ring_buffer(slot, vertices.buffer as usize, true);
                    let base = vertices.offset as usize;
                    let stride = layout.stride() as i32;
                    unsafe {
                        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
                        let float_at = |i: usize| (base + i * 4) as *const _;
                        match layout {
                            ImmLayout::D2 => {
                                // `Imm2DVertex`: pos (2), uv (2), color (4), p (4), q (4).
                                gl::BindVertexArray(self.binding.imm2d_vao);
                                for (location, components, at) in [
                                    (0u32, 2, 0usize),
                                    (2, 2, 2),
                                    (3, 4, 4),
                                    (IMM_PARAMS_LOCATION, 4, 8),
                                    (IMM_PARAMS2_LOCATION, 4, 12),
                                ] {
                                    gl::VertexAttribPointer(
                                        location,
                                        components,
                                        gl::FLOAT,
                                        gl::FALSE,
                                        stride,
                                        float_at(at),
                                    );
                                }
                            }
                            ImmLayout::D3 => {
                                // `Imm3DVertex`: pos (3), uv (2), color (4).
                                gl::BindVertexArray(self.binding.imm3d_vao);
                                for (location, components, at) in
                                    [(0u32, 3, 0usize), (2, 2, 3), (3, 4, 5)]
                                {
                                    gl::VertexAttribPointer(
                                        location,
                                        components,
                                        gl::FLOAT,
                                        gl::FALSE,
                                        stride,
                                        float_at(at),
                                    );
                                }
                            }
                        }
                        gl::DrawArrays(self.binding.topology, 0, *count as i32);
                        gl::BindVertexArray(0);
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    }
                    self.this_frame_stats.draw_immediate_calls += 1;
                    self.this_frame_stats.immediate_vertices += *count as u64;
                    self.this_frame_stats.vertices_drawn += *count as u64;
                    #[cfg(feature = "stats-server")]
                    self.count_draw();
                }
                PassCmd::DrawMeshInstanced {
                    mesh,
                    index_count,
                    instances,
                    count,
                } => {
                    let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(mesh) else {
                        warn!("DrawMeshInstanced: mesh {mesh:?} not found");
                        return;
                    };
                    let vao = *vao;
                    let buffer = self.ring_buffer(slot, instances.buffer as usize, true);
                    let base = instances.offset as usize;
                    // `InstanceData`: model matrix (4 columns, locations 4..7),
                    // color (8), scale (9); see `include/instanced.glsl`.
                    let stride = std::mem::size_of::<InstanceData>() as i32;
                    unsafe {
                        gl::BindVertexArray(vao);
                        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
                        for col in 0..4u32 {
                            let attrib = 4 + col;
                            gl::EnableVertexAttribArray(attrib);
                            gl::VertexAttribPointer(
                                attrib,
                                4,
                                gl::FLOAT,
                                gl::FALSE,
                                stride,
                                (base + col as usize * 16) as *const _,
                            );
                            gl::VertexAttribDivisor(attrib, 1);
                        }
                        gl::EnableVertexAttribArray(8);
                        gl::VertexAttribPointer(
                            8,
                            4,
                            gl::FLOAT,
                            gl::FALSE,
                            stride,
                            (base + 64) as *const _,
                        );
                        gl::VertexAttribDivisor(8, 1);
                        gl::EnableVertexAttribArray(9);
                        gl::VertexAttribPointer(
                            9,
                            1,
                            gl::FLOAT,
                            gl::FALSE,
                            stride,
                            (base + 80) as *const _,
                        );
                        gl::VertexAttribDivisor(9, 1);
                        gl::DrawElementsInstanced(
                            self.binding.topology,
                            *index_count as i32,
                            gl::UNSIGNED_INT,
                            std::ptr::null(),
                            *count as i32,
                        );
                        for attrib in 4..=9 {
                            gl::VertexAttribDivisor(attrib, 0);
                            gl::DisableVertexAttribArray(attrib);
                        }
                        gl::BindVertexArray(0);
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    }
                    self.this_frame_stats.draw_instanced_calls += 1;
                    self.this_frame_stats.instanced_data_items += *count as u64;
                    self.this_frame_stats.vertices_drawn += *index_count as u64 * *count as u64;
                    #[cfg(feature = "stats-server")]
                    self.count_draw();
                }
                PassCmd::DrawInstancedIndices {
                    mesh,
                    index_count,
                    indices,
                    count,
                } => {
                    let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(mesh) else {
                        warn!("DrawInstancedIndices: mesh {mesh:?} not found");
                        return;
                    };
                    let vao = *vao;
                    let buffer = self.ring_buffer(slot, indices.buffer as usize, true);
                    // Attribute 10: instance index (uint, divisor 1). Must use
                    // VertexAttribIPointer for integer attributes: the float
                    // path converts bits to floats and the shader's `in uint`
                    // reads garbage.
                    const ATTRIB: u32 = 10;
                    unsafe {
                        gl::BindVertexArray(vao);
                        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
                        gl::EnableVertexAttribArray(ATTRIB);
                        gl::VertexAttribIPointer(
                            ATTRIB,
                            1,
                            gl::UNSIGNED_INT,
                            0,
                            indices.offset as usize as *const _,
                        );
                        gl::VertexAttribDivisor(ATTRIB, 1);
                        gl::DrawElementsInstanced(
                            self.binding.topology,
                            *index_count as i32,
                            gl::UNSIGNED_INT,
                            std::ptr::null(),
                            *count as i32,
                        );
                        gl::VertexAttribDivisor(ATTRIB, 0);
                        gl::DisableVertexAttribArray(ATTRIB);
                        gl::BindVertexArray(0);
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    }
                    self.this_frame_stats.draw_instanced_calls += 1;
                    self.this_frame_stats.instanced_data_items += *count as u64;
                    self.this_frame_stats.vertices_drawn += *index_count as u64 * *count as u64;
                    #[cfg(feature = "stats-server")]
                    self.count_draw();
                }
            }
        }
    }

    #[cfg(feature = "stats-server")]
    fn count_draw(&mut self) {
        self.stats.draw_calls += 1;
        self.this_frame_stats.draw_calls += 1;
    }

    #[cfg(feature = "stats-server")]
    fn count_state_change(&mut self) {
        self.stats.state_changes += 1;
        self.this_frame_stats.state_changes += 1;
    }

    /// Texture destroyed: forget its caches and unbind it from the units.
    pub(super) fn forget_texture(&mut self, id: ResourceId) {
        self.binding.mip_ranges.remove(&id);
        self.binding.tex_descs.remove(&id);
    }

    /// Attachment key helper kept next to the pass state for symmetry with
    /// the FBO cache (`AttachKey`): sampling views never alias attachments.
    #[allow(dead_code)]
    fn attach_key(view: &TexView) -> AttachKey {
        AttachKey {
            id: view.tex,
            dim: view.dim,
            mip: view.base_mip,
        }
    }

    // -----------------------------------------------------------------
    // Shader link: apply the layout, reflect the blocks
    // -----------------------------------------------------------------

    /// Apply a linked program's `#group` layout: block binding points and
    /// fixed sampler units. Draws never touch these again.
    pub(super) fn apply_layout(program: u32, layout: &ShaderLayout) {
        unsafe {
            for block in &layout.blocks {
                let Ok(name) = std::ffi::CString::new(block.name.as_str()) else {
                    return;
                };
                let index = gl::GetUniformBlockIndex(program, name.as_ptr());
                if index != gl::INVALID_INDEX {
                    gl::UniformBlockBinding(program, index, block.binding());
                }
            }
            if layout.textures.is_empty() {
                return;
            }
            let mut previous = 0;
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut previous);
            gl::UseProgram(program);
            for tex in &layout.textures {
                let Ok(name) = std::ffi::CString::new(tex.name.as_str()) else {
                    return;
                };
                let location = gl::GetUniformLocation(program, name.as_ptr());
                if location >= 0 {
                    gl::Uniform1i(location, tex.unit() as i32);
                }
            }
            gl::UseProgram(previous as u32);
        }
    }

    /// Reflect every active uniform block of `program`.
    pub(super) fn reflect_blocks(program: u32) -> Vec<BlockLayout> {
        let mut out = Vec::new();
        unsafe {
            let mut block_count = 0;
            gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_BLOCKS, &mut block_count);
            for block in 0..block_count as u32 {
                let mut name_len = 0;
                gl::GetActiveUniformBlockiv(
                    program,
                    block,
                    gl::UNIFORM_BLOCK_NAME_LENGTH,
                    &mut name_len,
                );
                let mut name_buf = vec![0u8; name_len.max(1) as usize];
                let mut written = 0;
                gl::GetActiveUniformBlockName(
                    program,
                    block,
                    name_buf.len() as i32,
                    &mut written,
                    name_buf.as_mut_ptr() as *mut _,
                );
                let name =
                    String::from_utf8_lossy(&name_buf[..written.max(0) as usize]).into_owned();

                let mut size = 0;
                gl::GetActiveUniformBlockiv(program, block, gl::UNIFORM_BLOCK_DATA_SIZE, &mut size);
                let mut active = 0;
                gl::GetActiveUniformBlockiv(
                    program,
                    block,
                    gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
                    &mut active,
                );
                let mut indices = vec![0i32; active.max(0) as usize];
                if active > 0 {
                    gl::GetActiveUniformBlockiv(
                        program,
                        block,
                        gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
                        indices.as_mut_ptr(),
                    );
                }
                let uniform_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
                let query = |pname: u32| -> Vec<i32> {
                    let mut values = vec![0i32; uniform_indices.len()];
                    if !uniform_indices.is_empty() {
                        gl::GetActiveUniformsiv(
                            program,
                            uniform_indices.len() as i32,
                            uniform_indices.as_ptr(),
                            pname,
                            values.as_mut_ptr(),
                        );
                    }
                    values
                };
                let types = query(gl::UNIFORM_TYPE);
                let counts = query(gl::UNIFORM_SIZE);
                let offsets = query(gl::UNIFORM_OFFSET);
                let array_strides = query(gl::UNIFORM_ARRAY_STRIDE);
                let matrix_strides = query(gl::UNIFORM_MATRIX_STRIDE);

                let mut members = Vec::with_capacity(uniform_indices.len());
                for (i, &uniform) in uniform_indices.iter().enumerate() {
                    let mut buf = vec![0u8; 256];
                    let mut len = 0;
                    gl::GetActiveUniformName(
                        program,
                        uniform,
                        buf.len() as i32,
                        &mut len,
                        buf.as_mut_ptr() as *mut _,
                    );
                    let mut member_name =
                        String::from_utf8_lossy(&buf[..len.max(0) as usize]).into_owned();
                    if let Some(stripped) = member_name.strip_suffix("[0]") {
                        member_name = stripped.to_string();
                    }
                    // Instanced blocks report `Block.member`.
                    if let Some((_, member)) = member_name.split_once('.') {
                        member_name = member.to_string();
                    }
                    members.push(BlockMember {
                        name: member_name,
                        ty: glsl_type(types[i] as u32),
                        offset: offsets[i].max(0) as u32,
                        count: counts[i].max(1) as u32,
                        array_stride: array_strides[i].max(0) as u32,
                        matrix_stride: matrix_strides[i].max(0) as u32,
                    });
                }
                out.push(BlockLayout {
                    name,
                    size: size.max(0) as u32,
                    members,
                });
            }
        }
        out
    }
}

fn glsl_type(gl_type: u32) -> GlslType {
    match gl_type {
        gl::FLOAT => GlslType::Float,
        gl::FLOAT_VEC2 => GlslType::Vec2,
        gl::FLOAT_VEC3 => GlslType::Vec3,
        gl::FLOAT_VEC4 => GlslType::Vec4,
        gl::INT => GlslType::Int,
        gl::INT_VEC2 => GlslType::IVec2,
        gl::INT_VEC3 => GlslType::IVec3,
        gl::INT_VEC4 => GlslType::IVec4,
        gl::UNSIGNED_INT => GlslType::UInt,
        gl::UNSIGNED_INT_VEC2 => GlslType::UVec2,
        gl::UNSIGNED_INT_VEC3 => GlslType::UVec3,
        gl::UNSIGNED_INT_VEC4 => GlslType::UVec4,
        gl::BOOL => GlslType::Bool,
        gl::FLOAT_MAT3 => GlslType::Mat3,
        gl::FLOAT_MAT4 => GlslType::Mat4,
        _ => GlslType::Other,
    }
}

unsafe fn apply_blend(mode: BlendMode) {
    unsafe {
        match mode {
            BlendMode::Disabled => {
                gl::Disable(gl::BLEND);
                gl::BlendFunc(gl::ONE, gl::ZERO);
            }
            BlendMode::Alpha => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(
                    gl::SRC_ALPHA,
                    gl::ONE_MINUS_SRC_ALPHA,
                    gl::ONE,
                    gl::ONE_MINUS_SRC_ALPHA,
                );
            }
            BlendMode::Additive => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
            }
            BlendMode::PreMultAlpha => {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            }
        }
    }
}

unsafe fn apply_cull(face: CullFace) {
    unsafe {
        match face {
            CullFace::None => gl::Disable(gl::CULL_FACE),
            CullFace::Back => {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(gl::BACK);
            }
            CullFace::Front => {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(gl::FRONT);
            }
        }
    }
}

const _: () = assert!(GROUP_COUNT == 4);
