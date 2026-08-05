use std::collections::HashMap;

use glam::{IVec2, IVec3, IVec4, Vec2, Vec3, Vec4, ivec2, ivec3, ivec4, vec2, vec3, vec4};
use tracing::warn;

use super::{Tex1D, Tex2D, Tex3D, TexCube};
use crate::math::Matrix;
use crate::render::Renderer;

#[derive(Clone)]
pub enum ShaderVarData {
    Float(f32),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
    Int(i32),
    Int2(IVec2),
    Int3(IVec3),
    Int4(IVec4),
    Matrix(Matrix),
    Tex1D(Tex1D),
    Tex2D(Tex2D),
    Tex3D(Tex3D),
    TexCube(TexCube),
}

impl ShaderVarData {
    pub fn get_glsl_type(&self) -> &str {
        match self {
            ShaderVarData::Float(_) => "float",
            ShaderVarData::Float2(_) => "vec2",
            ShaderVarData::Float3(_) => "vec3",
            ShaderVarData::Float4(_) => "vec4",
            ShaderVarData::Int(_) => "int",
            ShaderVarData::Int2(_) => "ivec2",
            ShaderVarData::Int3(_) => "ivec3",
            ShaderVarData::Int4(_) => "ivec4",
            ShaderVarData::Matrix(_) => "mat4",
            ShaderVarData::Tex1D(_) => "sampler1D",
            ShaderVarData::Tex2D(_) => "sampler2D",
            ShaderVarData::Tex3D(_) => "sampler3D",
            ShaderVarData::TexCube(_) => "samplerCube",
        }
    }
}

pub struct ShaderVar;

/// The auto-var stack, owned by `Renderer` (was `static INST:
/// OnceLock<Mutex<ShaderVar>>`). This is CPU-only data - a name-keyed cache
/// that shaders consult when binding - but it's still main-thread-affine
/// state that has no business behind a global `Mutex`, and moving it here
/// drops the `unsafe impl Send for ShaderVarData` the old `Mutex<T: Send>`
/// bound required (nothing sends this across threads).
pub struct ShaderVarMap {
    var_map: HashMap<String, Vec<ShaderVarData>>,
}

impl ShaderVarMap {
    pub fn new() -> Self {
        Self {
            var_map: HashMap::with_capacity(16),
        }
    }

    /// Get the last element of the variable stack for this name, or None if it doesn't exist.
    pub fn get(&self, name: &str) -> Option<ShaderVarData> {
        self.var_map
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
    }

    fn push(&mut self, name: &str, data: ShaderVarData) {
        let stack = self.var_map.entry(name.into()).or_default();
        stack.push(data);
    }

    fn pop(&mut self, name: &str) {
        if let Some(stack) = self.var_map.get_mut(name) {
            if stack.pop().is_none() {
                warn!("Attempting to pop empty stack <{:?}>", name);
            }
        } else {
            warn!("Attempting to pop nonexisting stack <{:?}>", name);
        }
    }
}

impl Default for ShaderVarMap {
    fn default() -> Self {
        Self::new()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl ShaderVar {
    pub fn push_float(r: &mut Renderer, name: &str, x: f32) {
        r.data.shader_vars.push(name, ShaderVarData::Float(x));
    }

    pub fn push_float2(r: &mut Renderer, name: &str, x: f32, y: f32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Float2(vec2(x, y)));
    }

    pub fn push_float3(r: &mut Renderer, name: &str, x: f32, y: f32, z: f32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Float3(vec3(x, y, z)));
    }

    pub fn push_float4(r: &mut Renderer, name: &str, x: f32, y: f32, z: f32, w: f32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    pub fn push_int(r: &mut Renderer, name: &str, x: i32) {
        r.data.shader_vars.push(name, ShaderVarData::Int(x));
    }

    pub fn push_int2(r: &mut Renderer, name: &str, x: i32, y: i32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Int2(ivec2(x, y)));
    }

    pub fn push_int3(r: &mut Renderer, name: &str, x: i32, y: i32, z: i32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    pub fn push_int4(r: &mut Renderer, name: &str, x: i32, y: i32, z: i32, w: i32) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    pub fn push_matrix(r: &mut Renderer, name: &str, m: &Matrix) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Matrix(m.clone()));
    }

    pub fn push_tex1d(r: &mut Renderer, name: &str, t: &mut Tex1D) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Tex1D(t.clone()));
    }

    pub fn push_tex2d(r: &mut Renderer, name: &str, t: &mut Tex2D) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Tex2D(t.clone()));
    }

    pub fn push_tex3d(r: &mut Renderer, name: &str, t: &mut Tex3D) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::Tex3D(t.clone()));
    }

    pub fn push_tex_cube(r: &mut Renderer, name: &str, t: &mut TexCube) {
        r.data
            .shader_vars
            .push(name, ShaderVarData::TexCube(t.clone()));
    }

    pub fn pop(r: &mut Renderer, name: &str) {
        r.data.shader_vars.pop(name);
    }
}
