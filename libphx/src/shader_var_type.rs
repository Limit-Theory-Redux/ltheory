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

#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_FromStr(s: *const libc::c_char) -> ShaderVarType {
    let mut i: ShaderVarType = 0x1;
    while i <= 0xd {
        if s.convert() == ShaderVarType_GetGLSLName(i).convert() {
            return i;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn ShaderVarType_GetGLSLName(this: ShaderVarType) -> *const libc::c_char {
    match this {
        1 => return c_str!("float"),
        2 => return c_str!("vec2"),
        3 => return c_str!("vec3"),
        4 => return c_str!("vec4"),
        5 => return c_str!("int"),
        6 => return c_str!("ivec2"),
        7 => return c_str!("ivec3"),
        8 => return c_str!("ivec4"),
        9 => return c_str!("mat4"),
        10 => return c_str!("sampler1D"),
        11 => return c_str!("sampler2D"),
        12 => return c_str!("sampler3D"),
        13 => return c_str!("samplerCube"),
        _ => {}
    }
    std::ptr::null()
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
        1 => {
            return std::mem::size_of::<f32>() as i32;
        }
        2 => return std::mem::size_of::<Vec2>() as i32,
        3 => return std::mem::size_of::<Vec3>() as i32,
        4 => return std::mem::size_of::<Vec4>() as i32,
        5 => return std::mem::size_of::<i32>() as i32,
        6 => return std::mem::size_of::<IVec2>() as i32,
        7 => return std::mem::size_of::<IVec3>() as i32,
        8 => return std::mem::size_of::<IVec4>() as i32,
        9 => return std::mem::size_of::<*mut Matrix>() as i32,
        10 => return std::mem::size_of::<*mut Tex1D>() as i32,
        11 => return std::mem::size_of::<*mut Tex2D>() as i32,
        12 => return std::mem::size_of::<*mut Tex3D>() as i32,
        13 => {
            return std::mem::size_of::<*mut TexCube>() as i32;
        }
        _ => {}
    }
    0
}
