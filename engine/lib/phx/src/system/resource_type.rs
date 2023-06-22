use crate::{common::*, internal::static_string};

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
    resource_type_to_string(this)
        .map(|name| static_string!(name))
        .unwrap_or(std::ptr::null())
}

pub fn resource_type_to_string(this: ResourceType) -> Option<&'static str> {
    Some(match this {
        0 => "Font",
        1 => "Mesh",
        2 => "Other",
        3 => "Script",
        4 => "Shader",
        5 => "Sound",
        6 => "Tex1D",
        7 => "Tex2D",
        8 => "Tex3D",
        9 => "TexCube",
        _ => return None,
    })
}
