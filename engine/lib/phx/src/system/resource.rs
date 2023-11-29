use std::{ffi::CStr, path};

use internal::*;
use tracing::debug;

use super::*;
use crate::common::*;

static mut paths: [Vec<fn(String) -> String>; ResourceType_COUNT] = [
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
    Vec::new(),
];

#[inline]
unsafe extern "C" fn Resource_Resolve(
    ty: ResourceType,
    name: *const libc::c_char,
    fail_hard: bool,
) -> *const libc::c_char {
    resource_resolve(ty, &name.as_str(), fail_hard)
        .map(|val| static_string!(val))
        .unwrap_or(std::ptr::null())
}

unsafe fn resource_resolve(ty: ResourceType, name: &str, fail_hard: bool) -> Option<String> {
    for formatter in paths[ty as usize].iter() {
        let path = formatter(name.into());

        if file_exists(&path) {
            return Some(path);
        }
    }

    if !name.is_empty() && file_exists(name) as i32 != 0 {
        return Some(name.into());
    }

    if fail_hard {
        panic!(
            "Resource_Resolve: Failed to find {:?}:{ty} <{name}>. Current directory: {:?}",
            resource_type_to_string(ty),
            std::env::current_dir(),
        );
    }

    None
}

pub unsafe fn Resource_AddPath(ty: ResourceType, formatter: fn(String) -> String) {
    paths[ty as usize].push(formatter);
}

#[no_mangle]
pub unsafe extern "C" fn Resource_Exists(ty: ResourceType, name: *const libc::c_char) -> bool {
    !(Resource_Resolve(ty, name, false)).is_null()
}

#[no_mangle]
pub unsafe extern "C" fn Resource_GetPath(
    ty: ResourceType,
    name: *const libc::c_char,
) -> *const libc::c_char {
    Resource_Resolve(ty, name, true)
}

#[no_mangle]
pub unsafe extern "C" fn Resource_LoadBytes(
    ty: ResourceType,
    name: *const libc::c_char,
) -> *mut Bytes {
    let path = Resource_Resolve(ty, name, true);
    let data: *mut Bytes = File_ReadBytes(path);
    if data.is_null() {
        panic!(
            "Resource_LoadBytes: Failed to load {:?}:{} <{:?}> at <{:?}>",
            resource_type_to_string(ty),
            ty,
            CStr::from_ptr(name),
            CStr::from_ptr(path),
        );
    }
    data
}

#[no_mangle]
pub unsafe extern "C" fn Resource_LoadCstr(
    ty: ResourceType,
    name: *const libc::c_char,
) -> *const libc::c_char {
    resource_load_cstr(ty, &name.as_str())
        .map(|val| static_string!(val))
        .unwrap_or(std::ptr::null())
}

pub unsafe fn resource_load_cstr(ty: ResourceType, name: &str) -> Option<String> {
    let path = resource_resolve(ty, name, true)?;
    let data = file_read_cstr(&path)?;

    if data.is_empty() {
        panic!(
            "Resource_LoadCstr: Failed to load {:?}:{} <{name}> at <{path}>",
            resource_type_to_string(ty),
            ty,
        );
    }

    Some(data)
}

macro_rules! add_path {
    ($ty:ident, $p:literal) => {
        Resource_AddPath($ty, |s| format!($p, s));
    };
}

#[no_mangle]
pub unsafe extern "C" fn Resource_Init() {
    // TODO: this is needed for tests not to crash periodically. Should be removed after resource refactoring
    for path in &mut paths {
        path.clear();
    }

    add_path!(ResourceType_Font, "../shared/res/font/{}.ttf");
    add_path!(ResourceType_Font, "../shared/res/font/{}.otf");
    add_path!(ResourceType_Mesh, "../shared/res/mesh/{}.bin");
    add_path!(ResourceType_Mesh, "../shared/res/mesh/{}.obj");
    add_path!(ResourceType_Other, "../shared/res/{}");
    add_path!(ResourceType_Script, "../shared/res/script/{}.lua");
    add_path!(ResourceType_Shader, "../shared/res/shader/{}.glsl");
    add_path!(ResourceType_Sound, "../shared/res/sound/{}.mp3");
    add_path!(ResourceType_Sound, "../shared/res/sound/{}.ogg");
    add_path!(ResourceType_Sound, "../shared/res/sound/{}.ogx");
    add_path!(ResourceType_Sound, "../shared/res/sound/{}.wav");
    add_path!(ResourceType_Tex1D, "../shared/res/tex1d/{}.bin");
    add_path!(ResourceType_Tex2D, "../shared/res/tex2d/{}.jpg");
    add_path!(ResourceType_Tex2D, "../shared/res/tex2d/{}.png");
    add_path!(ResourceType_Tex3D, "../shared/res/tex3d/{}.bin");
    add_path!(ResourceType_TexCube, "../shared/res/texcube/{}");

    add_path!(ResourceType_Font, "./res/font/{}.ttf");
    add_path!(ResourceType_Font, "./res/font/{}.otf");
    add_path!(ResourceType_Mesh, "./res/mesh/{}.bin");
    add_path!(ResourceType_Mesh, "./res/mesh/{}.obj");
    add_path!(ResourceType_Other, "./res/{}");
    add_path!(ResourceType_Script, "./res/script/{}.lua");
    add_path!(ResourceType_Shader, "./res/shader/{}.glsl");
    add_path!(ResourceType_Sound, "./res/sound/{}.mp3");
    add_path!(ResourceType_Sound, "./res/sound/{}.ogg");
    add_path!(ResourceType_Sound, "./res/sound/{}.ogx");
    add_path!(ResourceType_Sound, "./res/sound/{}.wav");
    add_path!(ResourceType_Tex1D, "./res/tex1d/{}.bin");
    add_path!(ResourceType_Tex2D, "./res/tex2d/{}.jpg");
    add_path!(ResourceType_Tex2D, "./res/tex2d/{}.png");
    add_path!(ResourceType_Tex3D, "./res/tex3d/{}.bin");
    add_path!(ResourceType_TexCube, "./res/texcube/{}");

    add_path!(ResourceType_Font, "{}.ttf");
    add_path!(ResourceType_Font, "{}.otf");
    add_path!(ResourceType_Mesh, "{}.bin");
    add_path!(ResourceType_Mesh, "{}.obj");
    add_path!(ResourceType_Other, "{}");
    add_path!(ResourceType_Script, "{}.lua");
    add_path!(ResourceType_Shader, "{}.glsl");
    add_path!(ResourceType_Sound, "{}.mp3");
    add_path!(ResourceType_Sound, "{}.ogg");
    add_path!(ResourceType_Sound, "{}.ogx");
    add_path!(ResourceType_Sound, "{}.wav");
    add_path!(ResourceType_Tex1D, "{}.bin");
    add_path!(ResourceType_Tex2D, "{}.jpg");
    add_path!(ResourceType_Tex2D, "{}.png");
    add_path!(ResourceType_Tex3D, "{}.bin");
    add_path!(ResourceType_TexCube, "{}");
}
