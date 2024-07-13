use internal::*;

use super::*;
use crate::math::*;

struct ShaderVarInfo {
    glsl_name: &'static str,
    name: &'static str,
    size: i32,
}

impl ShaderVarInfo {
    const fn new<T>(glsl_name: &'static str, name: &'static str) -> Self {
        Self {
            glsl_name,
            name,
            size: std::mem::size_of::<T>() as i32,
        }
    }
}

const SHADER_VAR_INFO: [ShaderVarInfo; 13] = [
    ShaderVarInfo::new::<f32>("float", "float"),
    ShaderVarInfo::new::<Vec2>("vec2", "float2"),
    ShaderVarInfo::new::<Vec3>("vec3", "float3"),
    ShaderVarInfo::new::<Vec4>("vec4", "float4"),
    ShaderVarInfo::new::<i32>("int", "int"),
    ShaderVarInfo::new::<IVec2>("ivec2", "int2"),
    ShaderVarInfo::new::<IVec3>("ivec3", "int3"),
    ShaderVarInfo::new::<IVec4>("ivec4", "int4"),
    ShaderVarInfo::new::<*mut Matrix>("mat4", "Matrix"),
    ShaderVarInfo::new::<*mut Tex1D>("sampler1D", "Tex1D"),
    ShaderVarInfo::new::<*mut Tex2D>("sampler2D", "Tex2D"),
    ShaderVarInfo::new::<*mut Tex3D>("sampler3D", "Tex3D"),
    ShaderVarInfo::new::<*mut TexCube>("samplerCube", "TexCube"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ShaderVarType(i32);

impl std::fmt::Display for ShaderVarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for ShaderVarType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<usize> for ShaderVarType {
    fn from(value: usize) -> Self {
        Self(value as i32)
    }
}

impl ShaderVarType {
    pub const UNKNOWN: ShaderVarType = ShaderVarType(0);

    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl ShaderVarType {
    pub fn from_str(name: &str) -> ShaderVarType {
        SHADER_VAR_INFO
            .iter()
            .enumerate()
            .find(|(_, info)| name == info.glsl_name)
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
            .into()
    }

    pub fn get_glsl_name(this: ShaderVarType) -> Option<String> {
        SHADER_VAR_INFO
            .get(this.0 as usize - 1)
            .map(|name| name.glsl_name.into())
    }

    pub fn get_name(this: ShaderVarType) -> Option<String> {
        SHADER_VAR_INFO
            .get(this.0 as usize - 1)
            .map(|info| info.name.into())
    }

    pub fn get_size(this: ShaderVarType) -> i32 {
        SHADER_VAR_INFO
            .get(this.0 as usize - 1)
            .map(|info| info.size)
            .unwrap_or(0)
    }
}

pub fn ShaderVarType_FromStr(name: *const libc::c_char) -> ShaderVarType {
    ShaderVarType::from_str(&name.as_str())
}

pub fn ShaderVarType_GetGLSLName(this: ShaderVarType) -> *const libc::c_char {
    ShaderVarType::get_glsl_name(this)
        .map(|name| static_string!(name))
        .unwrap_or(std::ptr::null())
}

pub fn ShaderVarType_GetName(this: ShaderVarType) -> *const libc::c_char {
    ShaderVarType::get_name(this)
        .map(|name| static_string!(name))
        .unwrap_or(std::ptr::null())
}

pub fn ShaderVarType_GetSize(this: ShaderVarType) -> i32 {
    ShaderVarType::get_size(this)
}
