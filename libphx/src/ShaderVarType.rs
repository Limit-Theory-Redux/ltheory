use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Math::{IVec2, IVec3, IVec4, Vec2};
use crate::Matrix::*;
use crate::Tex1D::*;
use crate::Tex2D::*;
use crate::Tex3D::*;
use crate::TexCube::*;
use libc;
pub type ShaderVarType = i32;

#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_FromStr(mut s: *const libc::c_char) -> ShaderVarType {
    let mut i: ShaderVarType = 0x1;
    while i <= 0xd {
        if StrEqual(s, ShaderVarType_GetGLSLName(i)) {
            return i;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetGLSLName(mut this: ShaderVarType) -> *const libc::c_char {
    match this {
        1 => return b"float\0" as *const u8 as *const libc::c_char,
        2 => return b"vec2\0" as *const u8 as *const libc::c_char,
        3 => return b"vec3\0" as *const u8 as *const libc::c_char,
        4 => return b"vec4\0" as *const u8 as *const libc::c_char,
        5 => return b"int\0" as *const u8 as *const libc::c_char,
        6 => return b"ivec2\0" as *const u8 as *const libc::c_char,
        7 => return b"ivec3\0" as *const u8 as *const libc::c_char,
        8 => return b"ivec4\0" as *const u8 as *const libc::c_char,
        9 => return b"mat4\0" as *const u8 as *const libc::c_char,
        10 => return b"sampler1D\0" as *const u8 as *const libc::c_char,
        11 => return b"sampler2D\0" as *const u8 as *const libc::c_char,
        12 => return b"sampler3D\0" as *const u8 as *const libc::c_char,
        13 => return b"samplerCube\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetName(mut this: ShaderVarType) -> *const libc::c_char {
    match this {
        1 => return b"float\0" as *const u8 as *const libc::c_char,
        2 => return b"float2\0" as *const u8 as *const libc::c_char,
        3 => return b"float3\0" as *const u8 as *const libc::c_char,
        4 => return b"float4\0" as *const u8 as *const libc::c_char,
        5 => return b"int\0" as *const u8 as *const libc::c_char,
        6 => return b"int2\0" as *const u8 as *const libc::c_char,
        7 => return b"int3\0" as *const u8 as *const libc::c_char,
        8 => return b"int4\0" as *const u8 as *const libc::c_char,
        9 => return b"Matrix\0" as *const u8 as *const libc::c_char,
        10 => return b"Tex1D\0" as *const u8 as *const libc::c_char,
        11 => return b"Tex2D\0" as *const u8 as *const libc::c_char,
        12 => return b"Tex3D\0" as *const u8 as *const libc::c_char,
        13 => return b"TexCube\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetSize(mut this: ShaderVarType) -> i32 {
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
