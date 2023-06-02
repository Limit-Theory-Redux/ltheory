use crate::common::*;
use crate::math::Vec3;
use crate::math::Vec4;
use crate::math::{IVec2, IVec3, IVec4, Vec2};
use crate::matrix::*;
use crate::tex1d::*;
use crate::tex2d::*;
use crate::tex3d::*;
use crate::texcube::*;
use crate::*;

pub type ShaderVarType = i32;

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

#[no_mangle]
pub extern "C" fn ShaderVarType_FromStr(name: *const libc::c_char) -> ShaderVarType {
    shader_var_type_from_str(&name.convert())
}

pub fn shader_var_type_from_str(name: &str) -> ShaderVarType {
    SHADER_VAR_INFO
        .iter()
        .enumerate()
        .find(|(_, info)| name == info.glsl_name)
        .map(|(i, _)| i + 1)
        .unwrap_or(0) as ShaderVarType
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetGLSLName(this: ShaderVarType) -> *const libc::c_char {
    shader_var_type_get_glsl_name(this)
        .map(|name| static_string!(name))
        .unwrap_or(std::ptr::null())
}

pub fn shader_var_type_get_glsl_name(this: ShaderVarType) -> Option<String> {
    SHADER_VAR_INFO
        .get(this as usize - 1)
        .map(|name| name.glsl_name.into())
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetName(this: ShaderVarType) -> *const libc::c_char {
    SHADER_VAR_INFO
        .get(this as usize - 1)
        .map(|info| info.name.as_ptr() as *const libc::c_char)
        .unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetSize(this: ShaderVarType) -> i32 {
    SHADER_VAR_INFO
        .get(this as usize - 1)
        .map(|info| info.size)
        .unwrap_or(0)
}
