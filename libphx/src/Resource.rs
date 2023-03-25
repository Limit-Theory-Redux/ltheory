use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Common::*;
use crate::File::*;
use crate::Math::Vec3;
use crate::ResourceType::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PathElem {
    pub format: *const libc::c_char,
    pub next: *mut PathElem,
}

static mut paths: [*mut PathElem; 10] = [
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
];

#[inline]
unsafe extern "C" fn Resource_Resolve(
    type_0: ResourceType,
    name: *const libc::c_char,
    failhard: bool,
) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 256] = [0; 256];
    let mut elem: *mut PathElem = paths[type_0 as usize];
    while !elem.is_null() {
        let mut res: i32 = libc::snprintf(
            buffer.as_mut_ptr(),
            std::mem::size_of::<[libc::c_char; 256]>(),
            (*elem).format,
            name,
        );
        if res > 0 && res < std::mem::size_of::<[libc::c_char; 256]>() as libc::c_ulong as i32 {
            if File_Exists(buffer.as_mut_ptr() as *const libc::c_char) {
                return buffer.as_mut_ptr() as *const libc::c_char;
            }
        }
        elem = (*elem).next;
    }
    if !name.is_null() && File_Exists(name) as i32 != 0 {
        return name;
    }
    if failhard {
        Fatal(
            c_str!("Resource_Resolve: Failed to find %s <%s>"),
            ResourceType_ToString(type_0),
            name,
        );
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn Resource_AddPath(type_0: ResourceType, format: *const libc::c_char) {
    let mut this = MemNew!(PathElem);
    (*this).format = StrDup(format);
    (*this).next = paths[type_0 as usize];
    paths[type_0 as usize] = this;
}

#[no_mangle]
pub unsafe extern "C" fn Resource_Exists(type_0: ResourceType, name: *const libc::c_char) -> bool {
    !(Resource_Resolve(type_0, name, false)).is_null()
}

#[no_mangle]
pub unsafe extern "C" fn Resource_GetPath(
    type_0: ResourceType,
    name: *const libc::c_char,
) -> *const libc::c_char {
    Resource_Resolve(type_0, name, true)
}

#[no_mangle]
pub unsafe extern "C" fn Resource_LoadBytes(
    type_0: ResourceType,
    name: *const libc::c_char,
) -> *mut Bytes {
    let mut path: *const libc::c_char = Resource_Resolve(type_0, name, true);
    let mut data: *mut Bytes = File_ReadBytes(path);
    if data.is_null() {
        Fatal(
            c_str!("Resource_LoadBytes: Failed to load %s <%s> at <%s>"),
            ResourceType_ToString(type_0),
            name,
            path,
        );
    }
    data
}

#[no_mangle]
pub unsafe extern "C" fn Resource_LoadCstr(
    type_0: ResourceType,
    name: *const libc::c_char,
) -> *const libc::c_char {
    let mut path: *const libc::c_char = Resource_Resolve(type_0, name, true);
    let mut data: *const libc::c_char = File_ReadCstr(path);
    if data.is_null() {
        Fatal(
            c_str!("Resource_LoadCstr: Failed to load %s <%s> at <%s>"),
            ResourceType_ToString(type_0),
            name,
            path,
        );
    }
    data
}

#[no_mangle]
pub unsafe extern "C" fn Resource_Init() {
    Resource_AddPath(ResourceType_Font, c_str!("../shared/res/font/%s.ttf"));
    Resource_AddPath(ResourceType_Font, c_str!("../shared/res/font/%s.otf"));
    Resource_AddPath(ResourceType_Mesh, c_str!("../shared/res/mesh/%s.bin"));
    Resource_AddPath(ResourceType_Mesh, c_str!("../shared/res/mesh/%s.obj"));
    Resource_AddPath(ResourceType_Other, c_str!("../shared/res/%s"));
    Resource_AddPath(ResourceType_Script, c_str!("../shared/res/script/%s.lua"));
    Resource_AddPath(ResourceType_Shader, c_str!("../shared/res/shader/%s.glsl"));
    Resource_AddPath(ResourceType_Sound, c_str!("../shared/res/sound/%s.mp3"));
    Resource_AddPath(ResourceType_Sound, c_str!("../shared/res/sound/%s.ogg"));
    Resource_AddPath(ResourceType_Sound, c_str!("../shared/res/sound/%s.ogx"));
    Resource_AddPath(ResourceType_Sound, c_str!("../shared/res/sound/%s.wav"));
    Resource_AddPath(ResourceType_Tex1D, c_str!("../shared/res/tex1d/%s.bin"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("../shared/res/tex2d/%s.jpg"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("../shared/res/tex2d/%s.png"));
    Resource_AddPath(ResourceType_Tex3D, c_str!("../shared/res/tex3d/%s.bin"));
    Resource_AddPath(ResourceType_TexCube, c_str!("../shared/res/texcube/%s"));
    Resource_AddPath(ResourceType_Font, c_str!("./res/font/%s.ttf"));
    Resource_AddPath(ResourceType_Font, c_str!("./res/font/%s.otf"));
    Resource_AddPath(ResourceType_Mesh, c_str!("./res/mesh/%s.bin"));
    Resource_AddPath(ResourceType_Mesh, c_str!("./res/mesh/%s.obj"));
    Resource_AddPath(ResourceType_Other, c_str!("./res/%s"));
    Resource_AddPath(ResourceType_Script, c_str!("./res/script/%s.lua"));
    Resource_AddPath(ResourceType_Shader, c_str!("./res/shader/%s.glsl"));
    Resource_AddPath(ResourceType_Sound, c_str!("./res/sound/%s.mp3"));
    Resource_AddPath(ResourceType_Sound, c_str!("./res/sound/%s.ogg"));
    Resource_AddPath(ResourceType_Sound, c_str!("./res/sound/%s.ogx"));
    Resource_AddPath(ResourceType_Sound, c_str!("./res/sound/%s.wav"));
    Resource_AddPath(ResourceType_Tex1D, c_str!("./res/tex1d/%s.bin"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("./res/tex2d/%s.jpg"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("./res/tex2d/%s.png"));
    Resource_AddPath(ResourceType_Tex3D, c_str!("./res/tex3d/%s.bin"));
    Resource_AddPath(ResourceType_TexCube, c_str!("./res/texcube/%s"));
    Resource_AddPath(ResourceType_Font, c_str!("%s.ttf"));
    Resource_AddPath(ResourceType_Font, c_str!("%s.otf"));
    Resource_AddPath(ResourceType_Mesh, c_str!("%s.bin"));
    Resource_AddPath(ResourceType_Mesh, c_str!("%s.obj"));
    Resource_AddPath(ResourceType_Other, c_str!("%s"));
    Resource_AddPath(ResourceType_Script, c_str!("%s.lua"));
    Resource_AddPath(ResourceType_Shader, c_str!("%s.glsl"));
    Resource_AddPath(ResourceType_Sound, c_str!("%s.mp3"));
    Resource_AddPath(ResourceType_Sound, c_str!("%s.ogg"));
    Resource_AddPath(ResourceType_Sound, c_str!("%s.ogx"));
    Resource_AddPath(ResourceType_Sound, c_str!("%s.wav"));
    Resource_AddPath(ResourceType_Tex1D, c_str!("%s.bin"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("%s.jpg"));
    Resource_AddPath(ResourceType_Tex2D, c_str!("%s.png"));
    Resource_AddPath(ResourceType_Tex3D, c_str!("%s.bin"));
    Resource_AddPath(ResourceType_TexCube, c_str!("%s"));
}
