use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use glam::{ivec2, ivec3, ivec4, vec2, vec3, vec4};

use super::{ShaderState, ShaderVarData, Tex1D, Tex2D, Tex3D, TexCube, gl};
use crate::logging::{info, warn};
use crate::math::Matrix;
use crate::render::{Renderer, ResourceHandle, ResourceId};
use crate::rf::Rf;
use crate::system::{Profiler, Resource, ResourceType};

const INCLUDE_PATH: &str = "include/";

/// Counts uniform sends skipped by the per-shader value dedup (see
/// `ShaderShared::apply_uniform`). These are Lua→Rust FFI crossings that were
/// paid on the main thread but produced no render command; the stats dashboard
/// reads-and-resets this once per frame to show the hidden producer cost that
/// the command count doesn't capture.
static UNIFORM_DEDUP_SKIPS: AtomicU64 = AtomicU64::new(0);

/// Read-and-reset the dedup-skip counter (called once per frame by the stats
/// snapshot publisher on the main thread).
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
    auto_vars: Vec<ShaderAutoVar>,
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
    /// Revision of the shader-var stack at the last auto-var application. If
    /// unchanged at `start()`, the auto-var loop is skipped entirely - the
    /// stack values (camera matrices etc.) are identical to what this program
    /// already received, and GL uniform state persists across `glUseProgram`.
    /// Forced re-apply after reload via `u64::MAX`.
    last_auto_var_revision: u64,
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

#[derive(Clone, Default)]
struct GLSLCode {
    code: String,
    auto_vars: Vec<ShaderAutoVar>,
}

#[derive(Clone)]
struct ShaderAutoVar {
    type_name: String,
    name: String,
    index: gl::types::GLint,
}

impl GLSLCode {
    fn load(name: &str) -> GLSLCode {
        Self::preprocess(&Resource::load_string(ResourceType::Shader, name))
    }

    fn preprocess(code: &str) -> GLSLCode {
        let mut result = GLSLCode::default();

        for line in code.lines() {
            if let Some(include_val) = line.strip_prefix("#include ") {
                let path = format!("{INCLUDE_PATH}{include_val}");
                let mut include = Self::load(&path);

                result.code += &include.code;
                result.code += "\n";

                result.auto_vars.append(&mut include.auto_vars);
            } else if let Some(autovar_val) = line.strip_prefix("#autovar ") {
                Self::parse_autovar(autovar_val, &mut result.auto_vars);
            } else {
                result.code += line;
                result.code += "\n";
            }
        }

        result
    }

    fn parse_autovar(val: &str, auto_vars: &mut Vec<ShaderAutoVar>) {
        let line_tokens: Vec<_> = val.split(' ').collect();
        if line_tokens.len() == 2 {
            let var_type = line_tokens[0];
            let var_name = line_tokens[1];
            auto_vars.push(ShaderAutoVar {
                type_name: var_type.into(),
                name: var_name.into(),
                index: -1,
            });
        } else {
            warn!("Failed to parse autovar directive:\n  {val}");
        }
    }
}

impl Shader {
    fn from_preprocessed(
        r: &mut Renderer,
        name: String,
        vs_code: GLSLCode,
        mut fs_code: GLSLCode,
        vs_name: Option<String>,
        fs_name: Option<String>,
    ) -> Shader {
        // Combine autovars from all shaders.
        let mut auto_vars = vs_code.auto_vars;
        auto_vars.append(&mut fs_code.auto_vars);

        // Check for autovar conflicts.
        let mut auto_var_keys: HashSet<&str> = HashSet::new();
        for v in auto_vars.iter() {
            if auto_var_keys.contains(v.name.as_str()) {
                warn!("Shader <{}> contains duplicate #autovar <{}>", name, v.name);
                continue;
            };
            auto_var_keys.insert(v.name.as_str());
        }

        let (handle, auto_vars) = match create_shader_blocking(r, &vs_code.code, &fs_code.code) {
            Ok(handle) => (handle, auto_vars),
            Err(e) => {
                r.data.shader_errors.push(
                    &shader_error_key(&vs_name, &fs_name, &name),
                    "compile",
                    &e,
                );
                warn!("Shader '{name}' failed to compile, using error shader: {e}");
                // No autovars from a shader that doesn't declare any of them.
                (create_error_shader_handle(r), Vec::new())
            }
        };

        let mut shader = Shader {
            shared: Rf::new(ShaderShared {
                name,
                vs_name,
                fs_name,
                handle,
                auto_vars,
                uniform_location_cache: HashMap::new(),
                tex_index: 0,
                is_bound: false,
                pending_uniforms: vec![],
                last_auto_var_revision: u64::MAX,
                last_uniform_values: HashMap::new(),
            }),
        };
        shader.bind_auto_variables(r);
        shader
    }

    pub fn get_uniform_index(&self, r: &mut Renderer, name: &str) -> Option<gl::types::GLint> {
        let mut s = self.shared.as_mut();
        let id = s.handle.id();
        let index = resolve_uniform_location(r, id, name, &mut s.uniform_location_cache);
        if index >= 0 { Some(index) } else { None }
    }

    fn bind_auto_variables(&mut self, r: &mut Renderer) {
        let s = &mut *self.shared.as_mut();
        let id = s.handle.id();
        for var in s.auto_vars.iter_mut() {
            var.index = resolve_uniform_location(r, id, &var.name, &mut s.uniform_location_cache);
            if var.index < 0 {
                warn!(
                    "Automatic shader variable <{}> does not exist in shader {}",
                    var.name, s.name,
                )
            }
        }
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
        self.tex_index += 1;
        self.tex_index
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
        let mut fs_code = GLSLCode::load(&fs_name);

        // Try compile (non-panicking)
        let new_handle = match create_shader_blocking(r, &vs_code.code, &fs_code.code) {
            Ok(handle) => handle,
            Err(e) => {
                r.data
                    .shader_errors
                    .push(&format!("{vs_name}:{fs_name}"), "compile", &e);
                warn!("Shader '{name}' reload failed: {e}");
                return false;
            }
        };

        // Combine autovars
        let mut auto_vars = vs_code.auto_vars;
        auto_vars.append(&mut fs_code.auto_vars);

        // Deduplicate autovars
        let mut seen: HashSet<String> = HashSet::new();
        auto_vars.retain(|v| seen.insert(v.name.clone()));

        // Success — swap the resource handle in-place (all Rf clones see the
        // update); the old handle drops here, enqueuing its own destroy.
        {
            let s = &mut *self.shared.as_mut();
            s.handle = new_handle;
            s.auto_vars = auto_vars;
            s.uniform_location_cache.clear();
            s.pending_uniforms.clear();
            // New program: uniform values and locations all reset, so the
            // dedup cache must not suppress re-sending anything.
            s.last_uniform_values.clear();
            // Force auto-var re-application on next start() (new program).
            s.last_auto_var_revision = u64::MAX;
        }

        // Re-bind auto variables with new program
        self.bind_auto_variables(r);

        info!("Reloaded shader {}", name);
        true
    }

    pub fn name(&self) -> String {
        self.shared.as_ref().name.clone()
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
        self.shared.as_mut().tex_index = 0;
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

    /// Batched per-instance uniforms: mWorld, mWorldIT and scale in a single
    /// command instead of three separate SetUniform* commands. The instance
    /// values are unique per mesh (no dedup win), so the three GL uniform
    /// calls are batched on the render thread and the producer pays one
    /// command + one FFI crossing instead of three of each.
    #[bind(name = "ISetInstanceUniforms")]
    pub fn index_set_instance_uniforms(
        &mut self,
        r: &mut Renderer,
        world_index: i32,
        world_it_index: i32,
        scale_index: i32,
        world: &Matrix,
        world_it: &Matrix,
        scale: f32,
    ) {
        // Per-instance uniforms are unique per mesh (each entity has its
        // own mWorld/mWorldIT/scale), so the last_uniform_values dedup
        // cache cannot help here - it would clone 2 matrices + do 2
        // HashMap inserts per mesh for values that never match a previous
        // one. Skip the cache entirely and send the batched command
        // directly; the render thread clones into the command once.
        let mut shared = self.shared.as_mut();
        if !shared.is_bound {
            // Not bound: queue as three pending ops so the values are applied
            // on the next start() (same semantics as index_set_uniform).
            shared.pending_uniforms.push(SetUniformOp {
                index: world_index,
                data: ShaderVarData::Matrix(world.clone()),
            });
            shared.pending_uniforms.push(SetUniformOp {
                index: world_it_index,
                data: ShaderVarData::Matrix(world_it.clone()),
            });
            shared.pending_uniforms.push(SetUniformOp {
                index: scale_index,
                data: ShaderVarData::Float(scale),
            });
            return;
        }
        r.set_instance_uniforms(
            world_index,
            world_it_index,
            scale_index,
            world.to_cols_array(),
            world_it.to_cols_array(),
            scale,
        );
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
        s.tex_index = 0;

        // Apply pending uniforms.
        for p in std::mem::take(&mut s.pending_uniforms) {
            s.apply_uniform(r, p.index, &p.data);
        }

        // Fetch and bind automatic variables from the shader var stack.
        // The stack revision only bumps on push/pop; camera matrices are
        // pushed once per frame, so re-applying them per draw is redundant
        // (the values - and this program's uniform state - are unchanged).
        // SAFETY: only skip when this shader has NO *resolved* sampler
        // auto-vars - samplers must re-apply every start() because their
        // texture units are reallocated per draw and can be stolen by other
        // shaders. Unresolved samplers (index == -1, uniform absent from the
        // program, e.g. irMap/envMap on materials that don't use them) are
        // skipped by the loop below anyway and allocate no units, so they
        // must not disable the revision fast path - the hull shader is the
        // bulk of main-menu draws and would otherwise re-apply its full
        // auto-var stack per draw.
        let has_sampler_auto_vars = s
            .auto_vars
            .iter()
            .any(|v| v.type_name.starts_with("sampler") && v.index != -1);
        let stack_revision = r.data.shader_vars.revision();
        if has_sampler_auto_vars || stack_revision != s.last_auto_var_revision {
            for i in 0..s.auto_vars.len() {
                if s.auto_vars[i].index == -1 {
                    continue;
                }

                let Some(shader_var) = r.data.shader_vars.get(s.auto_vars[i].name.as_str()) else {
                    warn!(
                        "Shader variable stack does not contain variable <{}>",
                        s.auto_vars[i].name,
                    );
                    continue;
                };

                if shader_var.get_glsl_type() != s.auto_vars[i].type_name {
                    warn!(
                        "Attempting to get stack of type <{}> for shader variable <{}> when existing stack has type <{}>",
                        s.auto_vars[i].type_name,
                        s.auto_vars[i].name,
                        shader_var.get_glsl_type(),
                    );
                    continue;
                }

                s.index_set_uniform(r, s.auto_vars[i].index, shader_var);
            }
            s.last_auto_var_revision = stack_revision;
        }

        Profiler::end();
    }

    pub fn stop(&self, r: &mut Renderer) {
        self.shared.as_mut().is_bound = false;
        // NOTE: deliberately do NOT emit UnbindShader. glUseProgram(0) between
        // draws is protocol noise: the next shader's start() binds its own
        // program anyway, and GL 3.3 core has no fixed-function fallback that
        // would need program 0. Each unbind cost the main thread a ~2.6us
        // command send (~2k/frame in the main menu), and the executor's
        // current_program is only consulted for uniform-location resolution
        // while a shader is bound, so leaving the last program current is
        // safe. r is unused now but kept for signature stability.
        let _ = r;
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
/// the whole application. Deliberately has no autovars/uniforms - its only
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

    create_shader_blocking(r, ERROR_VS, ERROR_FS).expect("Failed to compile fallback error shader")
}

/// Compile+link a shader on the render thread and block for the result.
/// Mints a fresh `ResourceHandle` up front - on error it's simply dropped,
/// which harmlessly enqueues a destroy for a resource that was never created
/// (the executor's `DestroyResource` handler already no-ops on a missing id).
fn create_shader_blocking(
    r: &mut Renderer,
    vertex_src: &str,
    fragment_src: &str,
) -> Result<ResourceHandle, String> {
    let handle = r.create_resource();
    match r.create_shader(
        handle.id(),
        vertex_src.to_string(),
        fragment_src.to_string(),
    ) {
        None => Ok(handle),
        Some(err) => Err(err),
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
