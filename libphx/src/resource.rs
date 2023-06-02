use crate::bytes::*;
use crate::common::*;
use crate::file::*;
use crate::internal::*;

use crate::resource_type::*;

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
        CFatal!(
            "Resource_Resolve: Failed to find %s <%s>",
            ResourceType_ToString(ty),
            name,
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
    let path: *const libc::c_char = Resource_Resolve(ty, name, true);
    let data: *mut Bytes = File_ReadBytes(path);
    if data.is_null() {
        CFatal!(
            "Resource_LoadBytes: Failed to load %s <%s> at <%s>",
            ResourceType_ToString(ty),
            name,
            path,
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
        CFatal!(
            "Resource_LoadCstr: Failed to load %s <%s> at <%s>",
            ResourceType_ToString(ty),
            name,
            path,
        );
    }

    Some(data)
}

#[no_mangle]
pub unsafe extern "C" fn Resource_Init() {
    Resource_AddPath(ResourceType_Font, |s| {
        format!("../shared/res/font/{}.ttf", s)
    });
    Resource_AddPath(ResourceType_Font, |s| {
        format!("../shared/res/font/{}.otf", s)
    });
    Resource_AddPath(ResourceType_Mesh, |s| {
        format!("../shared/res/mesh/{}.bin", s)
    });
    Resource_AddPath(ResourceType_Mesh, |s| {
        format!("../shared/res/mesh/{}.obj", s)
    });
    Resource_AddPath(ResourceType_Other, |s| format!("../shared/res/{}", s));
    Resource_AddPath(ResourceType_Script, |s| {
        format!("../shared/res/script/{}.lua", s)
    });
    Resource_AddPath(ResourceType_Shader, |s| {
        format!("../shared/res/shader/{}.glsl", s)
    });
    Resource_AddPath(ResourceType_Sound, |s| {
        format!("../shared/res/sound/{}.mp3", s)
    });
    Resource_AddPath(ResourceType_Sound, |s| {
        format!("../shared/res/sound/{}.ogg", s)
    });
    Resource_AddPath(ResourceType_Sound, |s| {
        format!("../shared/res/sound/{}.ogx", s)
    });
    Resource_AddPath(ResourceType_Sound, |s| {
        format!("../shared/res/sound/{}.wav", s)
    });
    Resource_AddPath(ResourceType_Tex1D, |s| {
        format!("../shared/res/tex1d/{}.bin", s)
    });
    Resource_AddPath(ResourceType_Tex2D, |s| {
        format!("../shared/res/tex2d/{}.jpg", s)
    });
    Resource_AddPath(ResourceType_Tex2D, |s| {
        format!("../shared/res/tex2d/{}.png", s)
    });
    Resource_AddPath(ResourceType_Tex3D, |s| {
        format!("../shared/res/tex3d/{}.bin", s)
    });
    Resource_AddPath(ResourceType_TexCube, |s| {
        format!("../shared/res/texcube/{}", s)
    });
    Resource_AddPath(ResourceType_Font, |s| format!("./res/font/{}.ttf", s));
    Resource_AddPath(ResourceType_Font, |s| format!("./res/font/{}.otf", s));
    Resource_AddPath(ResourceType_Mesh, |s| format!("./res/mesh/{}.bin", s));
    Resource_AddPath(ResourceType_Mesh, |s| format!("./res/mesh/{}.obj", s));
    Resource_AddPath(ResourceType_Other, |s| format!("./res/{}", s));
    Resource_AddPath(ResourceType_Script, |s| format!("./res/script/{}.lua", s));
    Resource_AddPath(ResourceType_Shader, |s| format!("./res/shader/{}.glsl", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("./res/sound/{}.mp3", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("./res/sound/{}.ogg", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("./res/sound/{}.ogx", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("./res/sound/{}.wav", s));
    Resource_AddPath(ResourceType_Tex1D, |s| format!("./res/tex1d/{}.bin", s));
    Resource_AddPath(ResourceType_Tex2D, |s| format!("./res/tex2d/{}.jpg", s));
    Resource_AddPath(ResourceType_Tex2D, |s| format!("./res/tex2d/{}.png", s));
    Resource_AddPath(ResourceType_Tex3D, |s| format!("./res/tex3d/{}.bin", s));
    Resource_AddPath(ResourceType_TexCube, |s| format!("./res/texcube/{}", s));
    Resource_AddPath(ResourceType_Font, |s| format!("{}.ttf", s));
    Resource_AddPath(ResourceType_Font, |s| format!("{}.otf", s));
    Resource_AddPath(ResourceType_Mesh, |s| format!("{}.bin", s));
    Resource_AddPath(ResourceType_Mesh, |s| format!("{}.obj", s));
    Resource_AddPath(ResourceType_Other, |s| format!("{}", s));
    Resource_AddPath(ResourceType_Script, |s| format!("{}.lua", s));
    Resource_AddPath(ResourceType_Shader, |s| format!("{}.glsl", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("{}.mp3", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("{}.ogg", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("{}.ogx", s));
    Resource_AddPath(ResourceType_Sound, |s| format!("{}.wav", s));
    Resource_AddPath(ResourceType_Tex1D, |s| format!("{}.bin", s));
    Resource_AddPath(ResourceType_Tex2D, |s| format!("{}.jpg", s));
    Resource_AddPath(ResourceType_Tex2D, |s| format!("{}.png", s));
    Resource_AddPath(ResourceType_Tex3D, |s| format!("{}.bin", s));
    Resource_AddPath(ResourceType_TexCube, |s| format!("{}", s));
}
