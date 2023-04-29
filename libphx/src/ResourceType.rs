use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
pub type ResourceType = i32;

#[no_mangle]
pub static ResourceType_Font: ResourceType = 0;

#[no_mangle]
pub static ResourceType_Mesh: ResourceType = 0x1;

#[no_mangle]
pub static ResourceType_Other: ResourceType = 0x2;

#[no_mangle]
pub static ResourceType_Script: ResourceType = 0x3;

#[no_mangle]
pub static ResourceType_Shader: ResourceType = 0x4;

#[no_mangle]
pub static ResourceType_Sound: ResourceType = 0x5;

#[no_mangle]
pub static ResourceType_Tex1D: ResourceType = 0x6;

#[no_mangle]
pub static ResourceType_Tex2D: ResourceType = 0x7;

#[no_mangle]
pub static ResourceType_Tex3D: ResourceType = 0x8;

#[no_mangle]
pub static ResourceType_TexCube: ResourceType = 0x9;

pub const ResourceType_COUNT: usize = 10;

#[no_mangle]
pub extern "C" fn ResourceType_ToString(this: ResourceType) -> *const libc::c_char {
    match this {
        0 => return c_str!("Font"),
        1 => return c_str!("Mesh"),
        2 => return c_str!("Other"),
        3 => return c_str!("Script"),
        4 => return c_str!("Shader"),
        5 => return c_str!("Sound"),
        6 => return c_str!("Tex1D"),
        7 => return c_str!("Tex2D"),
        8 => return c_str!("Tex3D"),
        9 => return c_str!("TexCube"),
        _ => {}
    }
    std::ptr::null()
}
