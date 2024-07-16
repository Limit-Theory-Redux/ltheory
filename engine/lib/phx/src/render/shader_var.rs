use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use internal::*;

use super::*;
use crate::logging::*;
use crate::math::*;

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
    Tex1D(*mut Tex1D),
    Tex2D(*mut Tex2D),
    Tex3D(*mut Tex3D),
    TexCube(*mut TexCube),
}

// Trust me, it's fine.
unsafe impl Send for ShaderVarData {}

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

pub struct ShaderVar {
    var_map: HashMap<String, Vec<ShaderVarData>>,
}

impl ShaderVar {
    fn inst() -> MutexGuard<'static, ShaderVar> {
        static INST: OnceLock<Mutex<ShaderVar>> = OnceLock::new();
        INST.get_or_init(|| {
            Mutex::new(ShaderVar {
                var_map: HashMap::with_capacity(16),
            })
        })
        .lock()
        .unwrap()
    }

    /// Get the last element of the variable stack for this name, or None if it doesn't exist.
    pub fn get(name: &str) -> Option<ShaderVarData> {
        Self::inst()
            .var_map
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
    }

    /// Push an element to the variable stack for this name.
    fn push(name: &str, data: ShaderVarData) {
        let mut this = Self::inst();
        let stack = this.var_map.entry(name.into()).or_default();
        stack.push(data);
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl ShaderVar {
    pub fn push_float(name: &str, x: f32) {
        Self::push(name, ShaderVarData::Float(x));
    }

    pub fn push_float2(name: &str, x: f32, y: f32) {
        Self::push(name, ShaderVarData::Float2(vec2(x, y)));
    }

    pub fn push_float3(name: &str, x: f32, y: f32, z: f32) {
        Self::push(name, ShaderVarData::Float3(vec3(x, y, z)));
    }

    pub fn push_float4(name: &str, x: f32, y: f32, z: f32, w: f32) {
        Self::push(name, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    pub fn push_int(name: &str, x: i32) {
        Self::push(name, ShaderVarData::Int(x));
    }

    pub fn push_int2(name: &str, x: i32, y: i32) {
        Self::push(name, ShaderVarData::Int2(ivec2(x, y)));
    }

    pub fn push_int3(name: &str, x: i32, y: i32, z: i32) {
        Self::push(name, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    pub fn push_int4(name: &str, x: i32, y: i32, z: i32, w: i32) {
        Self::push(name, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    pub fn push_matrix(name: &str, m: &Matrix) {
        Self::push(name, ShaderVarData::Matrix(*m));
    }

    pub fn push_tex1d(name: &str, t: &mut Tex1D) {
        Self::push(name, ShaderVarData::Tex1D(t));
    }

    pub fn push_tex2d(name: &str, t: &mut Tex2D) {
        Self::push(name, ShaderVarData::Tex2D(t));
    }

    pub fn push_tex3d(name: &str, t: &mut Tex3D) {
        Self::push(name, ShaderVarData::Tex3D(t));
    }

    pub fn push_tex_cube(name: &str, t: &mut TexCube) {
        Self::push(name, ShaderVarData::TexCube(t));
    }

    pub fn pop(name: &str) {
        let mut this = Self::inst();
        if let Some(stack) = this.var_map.get_mut(name) {
            if stack.pop().is_none() {
                warn!("Attempting to pop empty stack <{:?}>", name);
            }
        } else {
            warn!("Attempting to pop nonexisting stack <{:?}>", name);
        }
    }
}
