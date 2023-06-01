use crate::common::*;
use crate::math::Vec3;
use crate::math::Vec4;
use crate::math::{IVec2, IVec3, IVec4, Vec2};
use crate::matrix::*;
use crate::tex1d::*;
use crate::tex2d::*;
use crate::tex3d::*;
use crate::tex_cube::*;
use crate::*;

pub type ShaderVarType = i32;

const SHADER_VAR_TYPE_GLSL_NAMES: [&str; 13] = [
    "float",
    "vec2",
    "vec3",
    "vec4",
    "int",
    "ivec2",
    "ivec3",
    "ivec4",
    "mat4",
    "sampler1D",
    "sampler2D",
    "sampler3D",
    "samplerCube",
];

#[no_mangle]
pub extern "C" fn ShaderVarType_FromStr(name: *const libc::c_char) -> ShaderVarType {
    shader_var_type_from_str(&name.convert())
}

pub fn shader_var_type_from_str(name: &str) -> ShaderVarType {
    SHADER_VAR_TYPE_GLSL_NAMES
        .iter()
        .enumerate()
        .find(|(_, n)| &name == *n)
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
    SHADER_VAR_TYPE_GLSL_NAMES
        .get(this as usize - 1)
        .map(|name| (*name).into())
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetName(this: ShaderVarType) -> *const libc::c_char {
    match this {
        1 => return c_str!("float"),
        2 => return c_str!("float2"),
        3 => return c_str!("float3"),
        4 => return c_str!("float4"),
        5 => return c_str!("int"),
        6 => return c_str!("int2"),
        7 => return c_str!("int3"),
        8 => return c_str!("int4"),
        9 => return c_str!("Matrix"),
        10 => return c_str!("Tex1D"),
        11 => return c_str!("Tex2D"),
        12 => return c_str!("Tex3D"),
        13 => return c_str!("TexCube"),
        _ => {}
    }
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetSize(this: ShaderVarType) -> i32 {
    match this {
        1 => std::mem::size_of::<f32>() as i32,
        2 => std::mem::size_of::<Vec2>() as i32,
        3 => std::mem::size_of::<Vec3>() as i32,
        4 => std::mem::size_of::<Vec4>() as i32,
        5 => std::mem::size_of::<i32>() as i32,
        6 => std::mem::size_of::<IVec2>() as i32,
        7 => std::mem::size_of::<IVec3>() as i32,
        8 => std::mem::size_of::<IVec4>() as i32,
        9 => std::mem::size_of::<*mut Matrix>() as i32,
        10 => std::mem::size_of::<*mut Tex1D>() as i32,
        11 => std::mem::size_of::<*mut Tex2D>() as i32,
        12 => std::mem::size_of::<*mut Tex3D>() as i32,
        13 => std::mem::size_of::<*mut TexCube>() as i32,
        _ => 0,
    }
}
