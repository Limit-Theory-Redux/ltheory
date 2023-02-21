use ::libc;
use glam::Vec3;
use glam::{IVec2, IVec3, IVec4, Vec2};
use crate::internal::Memory::*;
extern "C" {
    pub type Tex1D;
    pub type Tex2D;
    pub type Tex3D;
    pub type TexCube;
    pub type Matrix;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;




#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type ShaderVarType = int32;
#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_FromStr(mut s: cstr) -> ShaderVarType {
    let mut i: ShaderVarType = 0x1 as libc::c_int;
    while i <= 0xd as libc::c_int {
        if StrEqual(s, ShaderVarType_GetGLSLName(i)) {
            return i;
        }
        i += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetGLSLName(mut this: ShaderVarType) -> cstr {
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
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetName(mut this: ShaderVarType) -> cstr {
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
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVarType_GetSize(
    mut this: ShaderVarType,
) -> libc::c_int {
    match this {
        1 => {
            return ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
                as libc::c_int;
        }
        2 => return ::core::mem::size_of::<Vec2>() as libc::c_ulong as libc::c_int,
        3 => return ::core::mem::size_of::<Vec3>() as libc::c_ulong as libc::c_int,
        4 => return ::core::mem::size_of::<Vec4f>() as libc::c_ulong as libc::c_int,
        5 => return ::core::mem::size_of::<libc::c_int>() as libc::c_ulong as libc::c_int,
        6 => return ::core::mem::size_of::<IVec2>() as libc::c_ulong as libc::c_int,
        7 => return ::core::mem::size_of::<IVec3>() as libc::c_ulong as libc::c_int,
        8 => return ::core::mem::size_of::<IVec4>() as libc::c_ulong as libc::c_int,
        9 => return ::core::mem::size_of::<*mut Matrix>() as libc::c_ulong as libc::c_int,
        10 => return ::core::mem::size_of::<*mut Tex1D>() as libc::c_ulong as libc::c_int,
        11 => return ::core::mem::size_of::<*mut Tex2D>() as libc::c_ulong as libc::c_int,
        12 => return ::core::mem::size_of::<*mut Tex3D>() as libc::c_ulong as libc::c_int,
        13 => {
            return ::core::mem::size_of::<*mut TexCube>() as libc::c_ulong as libc::c_int;
        }
        _ => {}
    }
    return 0 as libc::c_int;
}
