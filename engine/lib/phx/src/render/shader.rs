use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "stats-server")]
use std::sync::atomic::{AtomicU64, Ordering};

use glam::{ivec2, ivec3, ivec4, vec2, vec3, vec4};

use super::{
    BlockLayout, DrawBlock, GROUP_COUNT, ShaderLayout, ShaderState, ShaderVarData, StageLayout,
    Tex1D, Tex2D, Tex3D, TexCube, TexDim, ViewBlock, gl,
};
use crate::logging::{info, warn};
use crate::math::Matrix;
use crate::render::{Renderer, ResourceHandle, ResourceId};
use crate::rf::Rf;
use crate::system::{Profiler, Resource, ResourceType};

const INCLUDE_PATH: &str = "include/";

/// The legacy per-draw texture-unit allocator hands out units above this one.
/// Units 0..2 belong to group 0 (the environment maps the passes bind at
/// every `beginPass`), so a legacy bind must never land there or the next
/// pass would clobber it. // S6: remove
const LEGACY_LAST_RESERVED_UNIT: gl::types::GLenum = 2;

/// Counts uniform sends skipped by the per-shader value dedup (see
/// `ShaderShared::apply_uniform`). These are Lua→Rust FFI crossings that were
/// paid on the main thread but produced no render command; the stats dashboard
/// reads-and-resets this once per frame to show the hidden producer cost that
/// the command count doesn't capture. Dashboard-only: gated with its only
/// reader so normal builds carry neither.
#[cfg(feature = "stats-server")]
static UNIFORM_DEDUP_SKIPS: AtomicU64 = AtomicU64::new(0);

/// Read-and-reset the dedup-skip counter (called once per frame by the stats
/// snapshot publisher on the main thread).
#[cfg(feature = "stats-server")]
pub(crate) fn uniform_dedup_skips() -> u64 {
    UNIFORM_DEDUP_SKIPS.swap(0, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct Shader {
    shared: Rf<ShaderShared>,
}

struct ShaderShared {
    name: String,
    vs_name: Option<String>,
    fs_name: Option<String>,
    handle: ResourceHandle,
    /// Bind-group layout recorded by the preprocessor (`#group` directives).
    layout: Arc<ShaderLayout>,
    /// Uniform blocks reflected at link time (empty for the error shader).
    blocks: Arc<Vec<BlockLayout>>,
    /// Bumped on every successful reload.
    generation: u32,
    /// Texture units claimed by `layout`; the legacy unit allocator skips
    /// them. // S6: remove
    fixed_units: u32,
    /// Rust-side cache of `get_uniform_index`/`GetVariable` lookups, so a
    /// name looked up more than once (e.g. `Shader::set_float("name", ..)`
    /// called every frame without a cached index) only pays the blocking
    /// `GetUniformLocationByResource` round-trip once. Separate from the
    /// executor's own per-program cache, which only serves the
    /// currently-bound program.
    uniform_location_cache: HashMap<Arc<str>, i32>,

    is_bound: bool,
    tex_index: gl::types::GLenum,
    pending_uniforms: Vec<SetUniformOp>,
    /// Last value sent for each uniform index, used to skip redundant
    /// uniform commands. The camera auto-vars (mView/mProj/etc.) are
    /// re-applied on every `start()` but rarely change, so without this
    /// the main thread re-sends thousands of identical uniform commands
    /// per frame. Sampler uniforms are never deduped (unit allocation).
    last_uniform_values: HashMap<i32, ShaderVarData>,
}

struct SetUniformOp {
    index: gl::types::GLint,
    data: ShaderVarData,
}

/// A preprocessed shader stage: `#include`s inlined, `#group` directives
/// stripped, and the `#group` declarations recorded.
#[derive(Clone, Default)]
pub(crate) struct GLSLCode {
    pub(crate) code: String,
    pub(crate) layout: StageLayout,
}

impl GLSLCode {
    fn load(name: &str) -> GLSLCode {
        Self::preprocess(&Resource::load_string(ResourceType::Shader, name))
    }

    fn preprocess(code: &str) -> GLSLCode {
        Self::preprocess_with(code, None, &mut |name| {
            Resource::load_string(ResourceType::Shader, name)
        })
    }

    /// `loader` receives include paths relative to the shader directory
    /// (`include/<name>`). `group` is the group in effect where this source
    /// is included; a `#group` directive inside an include does not leak out
    /// of it.
    pub(crate) fn preprocess_with(
        code: &str,
        group: Option<u8>,
        loader: &mut dyn FnMut(&str) -> String,
    ) -> GLSLCode {
        let mut result = GLSLCode::default();
        let mut group = group;

        for line in code.lines() {
            if let Some(include_val) = line.strip_prefix("#include ") {
                let path = format!("{INCLUDE_PATH}{include_val}");
                let include = Self::preprocess_with(&loader(&path), group, loader);

                result.code += &include.code;
                result.code += "\n";
                result.layout.blocks.extend(include.layout.blocks);
                result.layout.textures.extend(include.layout.textures);
                result.layout.errors.extend(include.layout.errors);
            } else if let Some(value) = line.strip_prefix("#group ") {
                match value.trim().parse::<u8>() {
                    Ok(g) if (g as usize) < GROUP_COUNT => group = Some(g),
                    _ => result
                        .layout
                        .errors
                        .push(format!("bad directive '{line}' (expected #group 0..3)")),
                }
            } else {
                if let Some(g) = group {
                    Self::record_declaration(line, g, &mut result.layout);
                }
                result.code += line;
                result.code += "\n";
            }
        }

        result
    }

    /// Record a uniform block or sampler declaration under `group`.
    fn record_declaration(line: &str, group: u8, layout: &mut StageLayout) {
        let t = line.trim();
        if t.starts_with("//") {
            return;
        }
        let Some(uniform_at) = t.find("uniform ") else {
            return;
        };
        // Only whole-line declarations: `layout(...) uniform ...` or
        // `uniform ...`.
        if uniform_at != 0 && !t.starts_with("layout") {
            return;
        }
        let rest = t[uniform_at + "uniform ".len()..].trim_start();
        if t.contains('{') {
            let name: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '{')
                .collect();
            if !name.is_empty() {
                layout.blocks.push((name, group));
            }
            return;
        }
        let mut tokens = rest.split_whitespace();
        let (Some(ty), Some(name)) = (tokens.next(), tokens.next()) else {
            return;
        };
        if !ty.contains("sampler") {
            return;
        }
        let name = name.trim_end_matches(';');
        match TexDim::from_sampler_type(ty) {
            Some(dim) => layout.textures.push((name.to_string(), group, dim)),
            None => layout.errors.push(format!(
                "unsupported sampler type '{ty}' for '{name}' in a #group"
            )),
        }
    }
}

impl Shader {
    fn from_preprocessed(
        r: &mut Renderer,
        name: String,
        vs_code: GLSLCode,
        fs_code: GLSLCode,
        vs_name: Option<String>,
        fs_name: Option<String>,
    ) -> Shader {
        let (layout, layout_errors) = ShaderLayout::merge(&[&vs_code.layout, &fs_code.layout]);
        let layout = Arc::new(layout);

        let compiled = if layout_errors.is_empty() {
            create_shader_blocking(r, &vs_code.code, &fs_code.code, &layout)
        } else {
            Err(format!(
                "invalid #group layout: {}",
                layout_errors.join("; ")
            ))
        };

        let (handle, layout, blocks) = match compiled {
            Ok((handle, blocks)) => {
                check_fixed_blocks(&name, &blocks);
                (handle, layout, blocks)
            }
            Err(e) => {
                r.data.shader_errors.push(
                    &shader_error_key(&vs_name, &fs_name, &name),
                    "compile",
                    &e,
                );
                warn!("Shader '{name}' failed to compile, using error shader: {e}");
                // The error shader declares nothing.
                (
                    create_error_shader_handle(r),
                    Arc::new(ShaderLayout::default()),
                    Vec::new(),
                )
            }
        };

        Shader {
            shared: Rf::new(ShaderShared {
                name,
                vs_name,
                fs_name,
                handle,
                fixed_units: layout.fixed_unit_mask(),
                layout,
                blocks: Arc::new(blocks),
                generation: 0,
                uniform_location_cache: HashMap::new(),
                tex_index: LEGACY_LAST_RESERVED_UNIT,
                is_bound: false,
                pending_uniforms: vec![],
                last_uniform_values: HashMap::new(),
            }),
        }
    }

    pub fn get_uniform_index(&self, r: &mut Renderer, name: &str) -> Option<gl::types::GLint> {
        let mut s = self.shared.as_mut();
        let id = s.handle.id();
        let index = resolve_uniform_location(r, id, name, &mut s.uniform_location_cache);
        if index >= 0 { Some(index) } else { None }
    }

    /// The bind-group layout recorded from the shader's `#group` directives.
    pub fn layout(&self) -> Arc<ShaderLayout> {
        self.shared.as_ref().layout.clone()
    }

    /// The uniform blocks reflected when the program was linked.
    pub fn blocks(&self) -> Arc<Vec<BlockLayout>> {
        self.shared.as_ref().blocks.clone()
    }

    /// The shader's GPU resource id.
    pub fn resource(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }

    pub fn set_uniform(&mut self, r: &mut Renderer, name: &str, data: ShaderVarData) {
        if let Some(index) = self.get_uniform_index(r, name) {
            self.index_set_uniform(r, index, data);
        }
    }

    pub fn index_set_uniform(&mut self, r: &mut Renderer, index: i32, data: ShaderVarData) {
        self.shared.as_mut().index_set_uniform(r, index, data);
    }
}

impl ShaderShared {
    // Increments the current texture index and returns the next free one.
    fn next_tex_index(&mut self) -> gl::types::GLenum {
        // S6: remove the skip. Units a `#group` layout fixed at link time
        // are not available to the legacy allocator.
        loop {
            self.tex_index += 1;
            if self.tex_index >= 32 || self.fixed_units & (1 << self.tex_index) == 0 {
                return self.tex_index;
            }
        }
    }

    pub fn index_set_uniform(&mut self, r: &mut Renderer, index: i32, data: ShaderVarData) {
        if self.is_bound {
            self.apply_uniform(r, index, &data);
        } else {
            self.pending_uniforms.push(SetUniformOp { index, data });
        }
    }

    pub fn apply_uniform(&mut self, r: &mut Renderer, index: i32, data: &ShaderVarData) {
        // Skip redundant uniform commands: the same value for the same
        // program keeps its GL state across start/stop cycles, so re-sending
        // it (e.g. camera auto-vars on every draw) is pure main-thread work.
        // Samplers are excluded - they allocate texture units on each set.
        let is_sampler = matches!(
            data,
            ShaderVarData::Tex1D(_)
                | ShaderVarData::Tex2D(_)
                | ShaderVarData::Tex3D(_)
                | ShaderVarData::TexCube(_)
        );
        if !is_sampler {
            if let Some(prev) = self.last_uniform_values.get(&index) {
                if prev.same_value(data) {
                    #[cfg(feature = "stats-server")]
                    UNIFORM_DEDUP_SKIPS.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            }
            self.last_uniform_values.insert(index, data.clone());
        }

        match data {
            ShaderVarData::Float(v) => {
                r.set_uniform_float(index, *v);
            }
            ShaderVarData::Float2(v) => {
                r.set_uniform_float2(index, v.x, v.y);
            }
            ShaderVarData::Float3(v) => {
                r.set_uniform_float3(index, v.x, v.y, v.z);
            }
            ShaderVarData::Float4(v) => {
                r.set_uniform_float4(index, v.x, v.y, v.z, v.w);
            }
            ShaderVarData::Int(v) => {
                r.set_uniform_int(index, *v);
            }
            ShaderVarData::Int2(v) => {
                r.set_uniform_int2(index, [v.x, v.y]);
            }
            ShaderVarData::Int3(v) => {
                r.set_uniform_int3(index, [v.x, v.y, v.z]);
            }
            ShaderVarData::Int4(v) => {
                r.set_uniform_int4(index, [v.x, v.y, v.z, v.w]);
            }
            ShaderVarData::Matrix(m) => {
                r.set_uniform_mat4(index, m.to_cols_array());
            }
            ShaderVarData::Tex1D(t) => {
                let tex_index = self.next_tex_index();

                r.set_uniform_int(index, tex_index as i32);
                r.bind_texture_1d_by_resource(tex_index, t.resource_id());
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();

                r.set_uniform_int(index, tex_index as i32);
                r.bind_texture_2d_by_resource(tex_index, t.resource_id());
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();

                r.set_uniform_int(index, tex_index as i32);
                r.bind_texture_3d_by_resource(tex_index, t.resource_id());
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();

                r.set_uniform_int(index, tex_index as i32);
                r.bind_texture_cube_by_resource(tex_index, t.resource_id());
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Shader {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, vs: &str, fs: &str) -> Shader {
        Self::from_preprocessed(
            r,
            "[anonymous shader]".into(),
            GLSLCode::preprocess(vs),
            GLSLCode::preprocess(fs),
            None,
            None,
        )
    }

    pub fn load(r: &mut Renderer, vs_name: &str, fs_name: &str) -> Shader {
        Self::from_preprocessed(
            r,
            format!("[vs: {vs_name}, fs: {fs_name}]"),
            GLSLCode::load(vs_name),
            GLSLCode::load(fs_name),
            Some(vs_name.to_string()),
            Some(fs_name.to_string()),
        )
    }

    /// Reload shader from disk. Returns true on success.
    /// On compile/link failure, keeps the old shader and returns false.
    pub fn reload(&mut self, r: &mut Renderer) -> bool {
        let s = self.shared.as_ref();
        let (Some(vs_name), Some(fs_name)) = (s.vs_name.clone(), s.fs_name.clone()) else {
            warn!("Cannot reload shader {} — no source paths stored", s.name);
            return false;
        };
        let name = s.name.clone();
        drop(s);

        // Reload and preprocess from disk
        let vs_code = GLSLCode::load(&vs_name);
        let fs_code = GLSLCode::load(&fs_name);
        let (layout, layout_errors) = ShaderLayout::merge(&[&vs_code.layout, &fs_code.layout]);
        let layout = Arc::new(layout);

        // Try compile (non-panicking)
        let compiled = if layout_errors.is_empty() {
            create_shader_blocking(r, &vs_code.code, &fs_code.code, &layout)
        } else {
            Err(format!(
                "invalid #group layout: {}",
                layout_errors.join("; ")
            ))
        };
        let (new_handle, blocks) = match compiled {
            Ok(result) => result,
            Err(e) => {
                r.data
                    .shader_errors
                    .push(&format!("{vs_name}:{fs_name}"), "compile", &e);
                warn!("Shader '{name}' reload failed: {e}");
                return false;
            }
        };
        check_fixed_blocks(&name, &blocks);

        // Success - swap the resource handle in-place (all Rf clones see the
        // update); the old handle drops here, enqueuing its own destroy.
        {
            let s = &mut *self.shared.as_mut();
            s.handle = new_handle;
            s.fixed_units = layout.fixed_unit_mask();
            s.layout = layout;
            s.blocks = Arc::new(blocks);
            s.generation += 1;
            s.uniform_location_cache.clear();
            s.pending_uniforms.clear();
            // New program: uniform values and locations all reset, so the
            // dedup cache must not suppress re-sending anything.
            s.last_uniform_values.clear();
        }

        info!("Reloaded shader {}", name);
        true
    }

    pub fn name(&self) -> String {
        self.shared.as_ref().name.clone()
    }

    /// A LuaJIT `ffi.typeof` struct declaration with the byte layout of the
    /// shader's uniform block `name` (empty if the shader has no such block).
    /// The Lua side wraps it as `shader:blockType(name)`.
    pub fn block_decl(&self, name: &str) -> String {
        self.shared
            .as_ref()
            .blocks
            .iter()
            .find(|b| b.name == name)
            .map(|b| b.lua_struct())
            .unwrap_or_default()
    }

    /// Size in bytes of the uniform block `name` (0 if absent).
    pub fn block_size(&self, name: &str) -> u32 {
        self.shared
            .as_ref()
            .blocks
            .iter()
            .find(|b| b.name == name)
            .map_or(0, |b| b.size)
    }

    /// Bumped each time hot reload relinks the shader, so cached block types
    /// can be regenerated.
    pub fn generation(&self) -> u32 {
        self.shared.as_ref().generation
    }

    /// The shader's GPU resource id (as a plain scalar - see
    /// `Renderer::add_entity`'s `mesh_id`/`shader_id` params for why this
    /// isn't `ResourceId` itself), e.g. for code that needs to reference the
    /// shader instead of calling `start`/`stop` itself (the batch API,
    /// `Renderer:addEntity`). Unlike `Mesh::resource_id`, this is a plain
    /// getter - `ShaderShared::handle` is always created eagerly in
    /// `new`/`from_preprocessed`, never lazily.
    pub fn resource_id(&self) -> u64 {
        self.shared.as_ref().handle.id().0
    }

    #[bind(name = "Clone")]
    pub fn acquire(&self) -> Shader {
        self.clone()
    }

    pub fn to_shader_state(&self) -> ShaderState {
        ShaderState::new(self)
    }

    #[bind(name = "GetVariable")]
    pub fn get_uniform_index_unchecked(&self, r: &mut Renderer, name: &str) -> i32 {
        self.get_uniform_index(r, name).unwrap_or_else(|| {
            panic!(
                "Shader <{}> has no variable <{}>",
                self.shared.as_ref().name,
                name,
            );
        })
    }

    pub fn has_variable(&self, r: &mut Renderer, name: &str) -> bool {
        self.get_uniform_index(r, name).is_some()
    }

    pub fn reset_tex_index(&mut self) {
        self.shared.as_mut().tex_index = LEGACY_LAST_RESERVED_UNIT;
    }

    pub fn set_float(&mut self, r: &mut Renderer, name: &str, value: f32) {
        self.set_uniform(r, name, ShaderVarData::Float(value));
    }

    #[bind(name = "ISetFloat")]
    pub fn index_set_float(&mut self, r: &mut Renderer, index: i32, value: f32) {
        self.index_set_uniform(r, index, ShaderVarData::Float(value));
    }

    pub fn set_float2(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32) {
        self.set_uniform(r, name, ShaderVarData::Float2(vec2(x, y)));
    }

    #[bind(name = "ISetFloat2")]
    pub fn index_set_float2(&mut self, r: &mut Renderer, index: i32, x: f32, y: f32) {
        self.index_set_uniform(r, index, ShaderVarData::Float2(vec2(x, y)));
    }

    pub fn set_float3(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32, z: f32) {
        self.set_uniform(r, name, ShaderVarData::Float3(vec3(x, y, z)));
    }

    #[bind(name = "ISetFloat3")]
    pub fn index_set_float3(&mut self, r: &mut Renderer, index: i32, x: f32, y: f32, z: f32) {
        self.index_set_uniform(r, index, ShaderVarData::Float3(vec3(x, y, z)));
    }

    pub fn set_float4(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32, z: f32, w: f32) {
        self.set_uniform(r, name, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    #[bind(name = "ISetFloat4")]
    pub fn index_set_float4(
        &mut self,
        r: &mut Renderer,
        index: i32,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) {
        self.index_set_uniform(r, index, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    pub fn set_int(&mut self, r: &mut Renderer, name: &str, value: i32) {
        self.set_uniform(r, name, ShaderVarData::Int(value));
    }

    #[bind(name = "ISetInt")]
    pub fn index_set_int(&mut self, r: &mut Renderer, index: i32, value: i32) {
        self.index_set_uniform(r, index, ShaderVarData::Int(value));
    }

    pub fn set_int2(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32) {
        self.set_uniform(r, name, ShaderVarData::Int2(ivec2(x, y)));
    }

    #[bind(name = "ISetInt2")]
    pub fn index_set_int2(&mut self, r: &mut Renderer, index: i32, x: i32, y: i32) {
        self.index_set_uniform(r, index, ShaderVarData::Int2(ivec2(x, y)));
    }

    pub fn set_int3(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32, z: i32) {
        self.set_uniform(r, name, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    #[bind(name = "ISetInt3")]
    pub fn index_set_int3(&mut self, r: &mut Renderer, index: i32, x: i32, y: i32, z: i32) {
        self.index_set_uniform(r, index, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    pub fn set_int4(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32, z: i32, w: i32) {
        self.set_uniform(r, name, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    #[bind(name = "ISetInt4")]
    pub fn index_set_int4(&mut self, r: &mut Renderer, index: i32, x: i32, y: i32, z: i32, w: i32) {
        self.index_set_uniform(r, index, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    pub fn set_matrix(&mut self, r: &mut Renderer, name: &str, value: &Matrix) {
        self.set_uniform(r, name, ShaderVarData::Matrix(value.clone()));
    }

    #[bind(name = "ISetMatrix")]
    pub fn index_set_matrix(&mut self, r: &mut Renderer, index: i32, value: &Matrix) {
        self.index_set_uniform(r, index, ShaderVarData::Matrix(value.clone()));
    }

    #[bind(name = "SetMatrixT")]
    pub fn set_matrix_transpose(&mut self, r: &mut Renderer, name: &str, value: &Matrix) {
        self.set_uniform(r, name, ShaderVarData::Matrix(value.transpose()));
    }

    #[bind(name = "ISetMatrixT")]
    pub fn index_set_matrix_transpose(&mut self, r: &mut Renderer, index: i32, value: &Matrix) {
        self.index_set_uniform(r, index, ShaderVarData::Matrix(value.transpose()));
    }

    pub fn set_tex1d(&mut self, r: &mut Renderer, name: &str, value: &mut Tex1D) {
        self.set_uniform(r, name, ShaderVarData::Tex1D(value.clone()));
    }

    #[bind(name = "ISetTex1D")]
    pub fn index_set_tex1d(&mut self, r: &mut Renderer, index: i32, value: &mut Tex1D) {
        self.index_set_uniform(r, index, ShaderVarData::Tex1D(value.clone()));
    }

    pub fn set_tex2d(&mut self, r: &mut Renderer, name: &str, value: &Tex2D) {
        self.set_uniform(r, name, ShaderVarData::Tex2D(value.clone()));
    }

    #[bind(name = "ISetTex2D")]
    pub fn index_set_tex2d(&mut self, r: &mut Renderer, index: i32, value: &mut Tex2D) {
        self.index_set_uniform(r, index, ShaderVarData::Tex2D(value.clone()));
    }

    pub fn set_tex3d(&mut self, r: &mut Renderer, name: &str, value: &mut Tex3D) {
        self.set_uniform(r, name, ShaderVarData::Tex3D(value.clone()));
    }

    #[bind(name = "ISetTex3D")]
    pub fn index_set_tex3d(&mut self, r: &mut Renderer, index: i32, value: &mut Tex3D) {
        self.index_set_uniform(r, index, ShaderVarData::Tex3D(value.clone()));
    }

    pub fn set_tex_cube(&mut self, r: &mut Renderer, name: &str, value: &mut TexCube) {
        self.set_uniform(r, name, ShaderVarData::TexCube(value.clone()));
    }

    #[bind(name = "ISetTexCube")]
    pub fn index_set_tex_cube(&mut self, r: &mut Renderer, index: i32, value: &mut TexCube) {
        self.index_set_uniform(r, index, ShaderVarData::TexCube(value.clone()));
    }

    // Singleton based shader functions - Old API.
    pub fn start(&mut self, r: &mut Renderer) {
        Profiler::begin("Shader_Start");

        let s = &mut *self.shared.as_mut();

        r.bind_shader_by_resource(s.handle.id(), None);
        s.is_bound = true;

        // Reset the tex index counter.
        s.tex_index = LEGACY_LAST_RESERVED_UNIT;

        // Apply pending uniforms.
        for p in std::mem::take(&mut s.pending_uniforms) {
            s.apply_uniform(r, p.index, &p.data);
        }
        Profiler::end();
    }

    pub fn stop(&self, _r: &mut Renderer) {
        self.shared.as_mut().is_bound = false;
        // NOTE: deliberately do NOT emit UnbindShader. glUseProgram(0) between
        // draws is protocol noise: the next shader's start() binds its own
        // program anyway, and GL 3.3 core has no fixed-function fallback that
        // would need program 0. Each unbind cost the main thread a ~2.6us
        // command send (~2k/frame in the main menu), and the executor's
        // current_program is only consulted for uniform-location resolution
        // while a shader is bound, so leaving the last program current is
        // safe.
    }
}

/// Key used to attribute a compile/reload error to a shader in the error
/// queue, matching the canonical `vs:fs` cache key used everywhere else
/// (`Cache.lua`, `ShaderWatcher`). Falls back to the shader's display name
/// for anonymous shaders (`Shader.Create`), which have no source paths.
fn shader_error_key(vs_name: &Option<String>, fs_name: &Option<String>, name: &str) -> String {
    match (vs_name, fs_name) {
        (Some(vs), Some(fs)) => format!("{vs}:{fs}"),
        _ => name.to_string(),
    }
}

/// Minimal magenta placeholder shader used when a shader fails to compile on
/// first load, so a broken shader renders visibly wrong instead of crashing
/// the whole application. Deliberately has no uniforms - its only
/// job is to compile and be unmistakably obvious on screen.
fn create_error_shader_handle(r: &mut Renderer) -> ResourceHandle {
    const ERROR_VS: &str = "#version 330\n\
        in vec3 vertex_position;\n\
        void main() {\n\
        \x20   gl_Position = vec4(vertex_position, 1.0);\n\
        }\n";
    const ERROR_FS: &str = "#version 330\n\
        out vec4 fragColor;\n\
        void main() {\n\
        \x20   fragColor = vec4(1.0, 0.0, 1.0, 1.0);\n\
        }\n";

    create_shader_blocking(r, ERROR_VS, ERROR_FS, &Arc::new(ShaderLayout::default()))
        .expect("Failed to compile fallback error shader")
        .0
}

/// Compile+link a shader on the render thread and block for the result,
/// which carries the uniform blocks reflected from the linked program.
/// Mints a fresh `ResourceHandle` up front - on error it's simply dropped,
/// which harmlessly enqueues a destroy for a resource that was never created
/// (the executor's `DestroyResource` handler already no-ops on a missing id).
fn create_shader_blocking(
    r: &mut Renderer,
    vertex_src: &str,
    fragment_src: &str,
    layout: &Arc<ShaderLayout>,
) -> Result<(ResourceHandle, Vec<BlockLayout>), String> {
    let handle = r.create_resource();
    let blocks = r.create_shader(
        handle.id(),
        vertex_src.to_string(),
        fragment_src.to_string(),
        layout.clone(),
    )?;
    Ok((handle, blocks))
}

/// Startup assert: the fixed Rust block structs must match what the linked
/// shader declares (size and member offsets), or every pass would feed the
/// GPU garbage.
fn check_fixed_blocks(shader_name: &str, blocks: &[BlockLayout]) {
    for block in blocks {
        if block.name == ViewBlock::NAME {
            if let Err(e) = ViewBlock::check_layout(block) {
                panic!("Shader '{shader_name}': {e}");
            }
        } else if block.name == DrawBlock::NAME {
            if let Err(e) = DrawBlock::check_layout(block) {
                panic!("Shader '{shader_name}': {e}");
            }
        }
    }
}

/// Blocking lookup of `name`'s uniform location for shader resource `id`,
/// checking the Rust-side `cache` first (see `ShaderShared::uniform_location_cache`).
fn resolve_uniform_location(
    r: &mut Renderer,
    id: ResourceId,
    name: &str,
    cache: &mut HashMap<Arc<str>, i32>,
) -> i32 {
    if let Some(&loc) = cache.get(name) {
        return loc;
    }

    let name: Arc<str> = Arc::from(name);
    let loc = r.get_uniform_location_by_resource(id, name.clone());
    cache.insert(name, loc);
    loc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pre(code: &str, includes: &[(&str, &str)]) -> GLSLCode {
        let includes: HashMap<String, String> = includes
            .iter()
            .map(|(n, c)| (format!("include/{n}"), c.to_string()))
            .collect();
        GLSLCode::preprocess_with(code, None, &mut |name| {
            includes
                .get(name)
                .unwrap_or_else(|| panic!("no include {name}"))
                .clone()
        })
    }

    #[test]
    fn group_directive_is_stripped_and_declarations_recorded() {
        let out = pre(
            "#version 330\n#group 1\nlayout(std140) uniform Mat {\n  vec4 c;\n};\nuniform sampler2D albedo;\n#group 3\nuniform samplerCube env;\nuniform float loose;\n",
            &[],
        );
        assert!(!out.code.contains("#group"));
        assert!(out.layout.errors.is_empty(), "{:?}", out.layout.errors);
        assert_eq!(out.layout.blocks, vec![("Mat".to_string(), 1)]);
        assert_eq!(
            out.layout.textures,
            vec![
                ("albedo".to_string(), 1, TexDim::D2),
                ("env".to_string(), 3, TexDim::Cube)
            ]
        );
    }

    #[test]
    fn declarations_without_a_group_are_not_recorded() {
        let out = pre(
            "uniform sampler2D a;\nlayout(std140) uniform B { float x; };\n",
            &[],
        );
        assert!(out.layout.blocks.is_empty() && out.layout.textures.is_empty());
    }

    #[test]
    fn includes_inherit_the_group_but_do_not_leak_theirs() {
        let out = pre(
            "#group 2\n#include inc\nuniform sampler2D after;\n",
            &[(
                "inc",
                "uniform sampler2D inherited;\n#group 0\nuniform sampler2D inner;\n",
            )],
        );
        assert_eq!(
            out.layout.textures,
            vec![
                ("inherited".to_string(), 2, TexDim::D2),
                ("inner".to_string(), 0, TexDim::D2),
                // the include's `#group 0` did not change the includer's group
                ("after".to_string(), 2, TexDim::D2),
            ]
        );
    }

    #[test]
    fn comments_and_non_declarations_are_ignored() {
        let out = pre(
            "#group 0\n// uniform sampler2D commented;\nfloat uniformity = 1.0;\nuniform sampler2D real; // trailing\n",
            &[],
        );
        assert_eq!(
            out.layout.textures,
            vec![("real".to_string(), 0, TexDim::D2)]
        );
    }

    #[test]
    fn bad_directives_and_sampler_types_are_errors() {
        let out = pre(
            "#group 9\n#group x\n#group 0\nuniform sampler2DShadow s;\n",
            &[],
        );
        assert_eq!(out.layout.errors.len(), 3, "{:?}", out.layout.errors);
    }

    #[test]
    fn merged_stage_layouts_number_by_first_appearance() {
        let vs = pre("#group 0\nlayout(std140) uniform V { vec4 a; };\n", &[]);
        let fs = pre(
            "#group 0\nlayout(std140) uniform V { vec4 a; };\nuniform samplerCube e;\nuniform samplerCube i;\n",
            &[],
        );
        let (layout, errors) = ShaderLayout::merge(&[&vs.layout, &fs.layout]);
        assert!(errors.is_empty());
        assert_eq!(layout.block("V").unwrap().binding(), 0);
        assert_eq!(layout.texture("e").unwrap().unit(), 0);
        assert_eq!(layout.texture("i").unwrap().unit(), 1);
    }
}
