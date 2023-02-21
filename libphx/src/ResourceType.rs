use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type cstr = *const libc::c_char;
pub type ResourceType = i32;
#[no_mangle]
pub static mut ResourceType_Font: ResourceType = 0 as i32;
#[no_mangle]
pub static mut ResourceType_Mesh: ResourceType = 0x1 as i32;
#[no_mangle]
pub static mut ResourceType_Other: ResourceType = 0x2 as i32;
#[no_mangle]
pub static mut ResourceType_Script: ResourceType = 0x3 as i32;
#[no_mangle]
pub static mut ResourceType_Shader: ResourceType = 0x4 as i32;
#[no_mangle]
pub static mut ResourceType_Sound: ResourceType = 0x5 as i32;
#[no_mangle]
pub static mut ResourceType_Tex1D: ResourceType = 0x6 as i32;
#[no_mangle]
pub static mut ResourceType_Tex2D: ResourceType = 0x7 as i32;
#[no_mangle]
pub static mut ResourceType_Tex3D: ResourceType = 0x8 as i32;
#[no_mangle]
pub static mut ResourceType_TexCube: ResourceType = 0x9 as i32;
#[no_mangle]
pub unsafe extern "C" fn ResourceType_ToString(mut this: ResourceType) -> cstr {
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
    return 0 as cstr;
}
