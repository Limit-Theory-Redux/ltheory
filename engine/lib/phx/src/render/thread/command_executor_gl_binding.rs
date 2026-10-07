//! GL 3.3 implementation of the binding model (doc/engine/render-api-v2.md,
//! section 4): pipelines with a diffed state cache, sampler objects, bind
//! groups, the uniform ring buffers, pass command execution, and the link-time
//! application and reflection of a shader's `#group` layout.

#![allow(unsafe_code)]

use std::collections::HashMap;

use tracing::{debug, error, warn};

use super::{AttachKey, GpuResource, MAX_TEXTURE_SLOTS, TextureBinding, TextureType};
use crate::render::gl;
use crate::render::{
    BindEntry, BindGroupId, BlendMode, BlockLayout, BlockMember, CommandCategory, CommandExecutor,
    CompareFn, CullFace, GROUP_COUNT, GROUP_DRAW, GROUP_FRAME, GROUP_INPUTS, GlslType, MAX_INPUTS,
    PassCmd, PassCommands, PipelineDesc, PipelineId, PolygonMode, ResourceId, SamplerDesc,
    SamplerId, Samplers, ShaderLayout, TexView, UNIFORM_ALIGN, ViewDim, block_binding, entry_unit,
    texture_unit,
};

/// Uniform block binding points (`group * 4 + k`) the passes drive.
const VIEW_BINDING: u32 = block_binding(GROUP_FRAME, 0);
const DRAW_BINDING: u32 = block_binding(GROUP_DRAW, 0);
const UBO_BINDINGS: usize = 16;
/// Texture units of the group-0 environment maps (`envMap`, `irMap`).
const ENV_UNIT: u32 = texture_unit(GROUP_FRAME, 0);
const IR_UNIT: u32 = texture_unit(GROUP_FRAME, 1);
/// Pass inputs start at the first unit of group 3.
const INPUT_UNIT: u32 = texture_unit(GROUP_INPUTS, 0);

/// What the executor believes GL's fixed-function state is. `None` = unknown.
/// Pipelines diff against it; the legacy state commands write through it so
/// the two paths can mix in one pass. // S6: remove the write-through
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
    pub gl_state: GlStateCache,
    /// The pipeline whose program and state are applied (`None` after any
    /// legacy shader or state command).
    pub current_pipeline: Option<PipelineId>,
    /// Topology of the current pipeline's draws.
    pub topology: u32,
    /// GL uniform buffers of the ring, per frame slot and chunk index.
    pub ring_buffers: Vec<Vec<u32>>,
    /// Frame (swap count) each ring buffer was last orphaned in.
    pub ring_epochs: Vec<Vec<u64>>,
    pub ubo_bound: [UboBinding; UBO_BINDINGS],
    /// Sampler object bound to each texture unit (0 = none).
    pub unit_samplers: [u32; MAX_TEXTURE_SLOTS],
    /// Mip range last set on each texture (`TEXTURE_BASE_LEVEL`, `MAX_LEVEL`).
    pub mip_ranges: HashMap<ResourceId, (i32, i32)>,
    /// Unit quad for `DrawFullscreen`.
    pub fullscreen_vao: u32,
    pub fullscreen_vbo: u32,
}

impl GlBindingState {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            samplers: HashMap::new(),
            bind_groups: HashMap::new(),
            gl_state: GlStateCache::default(),
            current_pipeline: None,
            topology: gl::TRIANGLES,
            ring_buffers: (0..crate::render::MAX_FRAMES_IN_FLIGHT)
                .map(|_| Vec::new())
                .collect(),
            ring_epochs: (0..crate::render::MAX_FRAMES_IN_FLIGHT)
                .map(|_| Vec::new())
                .collect(),
            ubo_bound: [UboBinding::default(); UBO_BINDINGS],
            unit_samplers: [0; MAX_TEXTURE_SLOTS],
            mip_ranges: HashMap::new(),
            fullscreen_vao: 0,
            fullscreen_vbo: 0,
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

    // -----------------------------------------------------------------
    // Uniform ring
    // -----------------------------------------------------------------

    fn ring_buffer(&mut self, slot: usize, index: usize) -> u32 {
        let epoch = self.stats.frame_count + 1;
        let buffers = &mut self.binding.ring_buffers[slot];
        let epochs = &mut self.binding.ring_epochs[slot];
        while buffers.len() <= index {
            let mut buffer = 0;
            unsafe {
                gl::GenBuffers(1, &mut buffer);
                gl::BindBuffer(gl::UNIFORM_BUFFER, buffer);
                gl::BufferData(
                    gl::UNIFORM_BUFFER,
                    crate::render::CHUNK_SIZE as isize,
                    std::ptr::null(),
                    gl::STREAM_DRAW,
                );
            }
            buffers.push(buffer);
            epochs.push(epoch);
        }
        // First use in a frame: orphan the old storage so the driver does not
        // have to wait for (or copy around) draws of an earlier frame that
        // still read it.
        if epochs[index] != epoch {
            epochs[index] = epoch;
            unsafe {
                gl::BindBuffer(gl::UNIFORM_BUFFER, buffers[index]);
                gl::BufferData(
                    gl::UNIFORM_BUFFER,
                    crate::render::CHUNK_SIZE as isize,
                    std::ptr::null(),
                    gl::STREAM_DRAW,
                );
            }
        }
        buffers[index]
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

    pub(super) fn cmd_pass_commands(&mut self, commands: &PassCommands) {
        let _sa = self.record_command(CommandCategory::Draw, false, false);
        if !self.has_gl_context() {
            return;
        }
        let slot = commands.slot as usize % self.binding.ring_buffers.len();
        check_gl(&"commands before this PassCommands");

        // Uploads first: everything the commands reference is in these.
        for chunk in &commands.uniforms {
            let buffer = self.ring_buffer(slot, chunk.at.buffer as usize);
            unsafe {
                gl::BindBuffer(gl::UNIFORM_BUFFER, buffer);
                gl::BufferSubData(
                    gl::UNIFORM_BUFFER,
                    chunk.at.offset as isize,
                    chunk.bytes.len() as isize,
                    chunk.bytes.as_ptr() as *const _,
                );
            }
        }

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
                    let Some(bg) = self.binding.bind_groups.get(id) else {
                        error!("SetBindGroup: bind group {id:?} was never created");
                        return;
                    };
                    let entries = bg.entries.clone();
                    let group = *group;
                    debug_assert_eq!(group, bg.group);
                    for entry in &entries {
                        let BindEntry::Texture { view, sampler, .. } = entry;
                        self.bind_view(entry_unit(group, entry), view, *sampler);
                    }
                }
                PassCmd::SetView { block, size } => {
                    let buffer = self.ring_buffer(slot, block.buffer as usize);
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
                    let buffer = self.ring_buffer(slot, at.buffer as usize);
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

    // -----------------------------------------------------------------
    // Write-through hooks for the legacy commands. // S6: remove
    // -----------------------------------------------------------------

    /// A legacy command changed the program or GL state behind the current
    /// pipeline; the next `SetPipeline` reapplies it in full.
    pub(super) fn invalidate_pipeline(&mut self) {
        self.binding.current_pipeline = None;
    }

    /// A legacy bind is about to put a texture on `slot`: the sampler object
    /// the passes left there would override the texture's own parameters.
    pub(super) fn drop_unit_sampler(&mut self, slot: usize) {
        if slot < MAX_TEXTURE_SLOTS && self.binding.unit_samplers[slot] != 0 {
            unsafe {
                gl::BindSampler(slot as u32, 0);
            }
            self.binding.unit_samplers[slot] = 0;
        }
    }

    /// A legacy command set `id`'s mip range directly.
    pub(super) fn note_mip_range(&mut self, id: ResourceId, min_level: i32, max_level: i32) {
        self.binding.mip_ranges.insert(id, (min_level, max_level));
    }

    /// Texture destroyed: forget its caches and unbind it from the units.
    pub(super) fn forget_texture(&mut self, id: ResourceId) {
        self.binding.mip_ranges.remove(&id);
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

    /// Which GL block binding points the legacy executor still sets by name.
    /// (`LightUBO` is declared without a `#group` until S5.) // S6: remove
    pub(super) fn bind_legacy_blocks(program: u32) {
        unsafe {
            let light = gl::GetUniformBlockIndex(program, c"LightUBO".as_ptr() as *const _);
            if light != gl::INVALID_INDEX {
                gl::UniformBlockBinding(program, light, crate::render::LIGHT_UBO_BINDING);
            }
        }
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
