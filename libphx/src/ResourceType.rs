use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type ResourceType = i32;

#[no_mangle]
pub static ResourceType_Font: ResourceType = 0_i32;

#[no_mangle]
pub static ResourceType_Mesh: ResourceType = 0x1_i32;

#[no_mangle]
pub static ResourceType_Other: ResourceType = 0x2_i32;

#[no_mangle]
pub static ResourceType_Script: ResourceType = 0x3_i32;

#[no_mangle]
pub static ResourceType_Shader: ResourceType = 0x4_i32;

#[no_mangle]
pub static ResourceType_Sound: ResourceType = 0x5_i32;

#[no_mangle]
pub static ResourceType_Tex1D: ResourceType = 0x6_i32;

#[no_mangle]
pub static ResourceType_Tex2D: ResourceType = 0x7_i32;

#[no_mangle]
pub static ResourceType_Tex3D: ResourceType = 0x8_i32;

#[no_mangle]
pub static ResourceType_TexCube: ResourceType = 0x9_i32;

#[no_mangle]
pub unsafe extern "C" fn ResourceType_ToString(mut this: ResourceType) -> *const libc::c_char {
    match this {
        0 => return b"Font\0" as *const u8 as *const libc::c_char,
        1 => return b"Mesh\0" as *const u8 as *const libc::c_char,
        2 => return b"Other\0" as *const u8 as *const libc::c_char,
        3 => return b"Script\0" as *const u8 as *const libc::c_char,
        4 => return b"Shader\0" as *const u8 as *const libc::c_char,
        5 => return b"Sound\0" as *const u8 as *const libc::c_char,
        6 => return b"Tex1D\0" as *const u8 as *const libc::c_char,
        7 => return b"Tex2D\0" as *const u8 as *const libc::c_char,
        8 => return b"Tex3D\0" as *const u8 as *const libc::c_char,
        9 => return b"TexCube\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    std::ptr::null()
}