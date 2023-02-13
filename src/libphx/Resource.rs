use ::libc;
use crate::internal::Memory::*;
use crate::ResourceType::*;

extern "C" {
    pub type Bytes;
    fn Fatal(_: cstr, _: ...);
    fn File_Exists(path: cstr) -> bool;
    fn File_ReadBytes(path: cstr) -> *mut Bytes;
    fn File_ReadCstr(path: cstr) -> cstr;
    fn ResourceType_ToString(_: ResourceType) -> cstr;
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type ResourceType = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PathElem {
    pub format: cstr,
    pub next: *mut PathElem,
}

static mut paths: [*mut PathElem; 10] = [
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
    0 as *const PathElem as *mut PathElem,
];
#[inline]
unsafe extern "C" fn Resource_Resolve(
    mut type_0: ResourceType,
    mut name: cstr,
    mut failhard: bool,
) -> cstr {
    static mut buffer: [libc::c_char; 256] = [0; 256];
    let mut elem: *mut PathElem = paths[type_0 as usize];
    while !elem.is_null() {
        let mut res: libc::c_int = snprintf(
            buffer.as_mut_ptr(),
            ::core::mem::size_of::<[libc::c_char; 256]>(),
            (*elem).format,
            name,
        );
        if res > 0 as libc::c_int
            && res
                < ::core::mem::size_of::<[libc::c_char; 256]>() as libc::c_ulong
                    as libc::c_int
        {
            if File_Exists(buffer.as_mut_ptr() as cstr) {
                return buffer.as_mut_ptr() as cstr;
            }
        }
        elem = (*elem).next;
    }
    if !name.is_null() && File_Exists(name) as libc::c_int != 0 {
        return name;
    }
    if failhard {
        Fatal(
            b"Resource_Resolve: Failed to find %s <%s>\0" as *const u8
                as *const libc::c_char,
            ResourceType_ToString(type_0),
            name,
        );
    }
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Resource_AddPath(mut type_0: ResourceType, mut format: cstr) {
    let mut self_0: *mut PathElem = MemAlloc(
        ::core::mem::size_of::<PathElem>() as usize,
    ) as *mut PathElem;
    (*self_0).format = StrDup(format);
    (*self_0).next = paths[type_0 as usize];
    paths[type_0 as usize] = self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Resource_Exists(
    mut type_0: ResourceType,
    mut name: cstr,
) -> bool {
    return !(Resource_Resolve(type_0, name, 0 as libc::c_int != 0)).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn Resource_GetPath(
    mut type_0: ResourceType,
    mut name: cstr,
) -> cstr {
    return Resource_Resolve(type_0, name, 1 as libc::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn Resource_LoadBytes(
    mut type_0: ResourceType,
    mut name: cstr,
) -> *mut Bytes {
    let mut path: cstr = Resource_Resolve(type_0, name, 1 as libc::c_int != 0);
    let mut data: *mut Bytes = File_ReadBytes(path);
    if data.is_null() {
        Fatal(
            b"Resource_LoadBytes: Failed to load %s <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
            ResourceType_ToString(type_0),
            name,
            path,
        );
    }
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Resource_LoadCstr(
    mut type_0: ResourceType,
    mut name: cstr,
) -> cstr {
    let mut path: cstr = Resource_Resolve(type_0, name, 1 as libc::c_int != 0);
    let mut data: cstr = File_ReadCstr(path);
    if data.is_null() {
        Fatal(
            b"Resource_LoadCstr: Failed to load %s <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
            ResourceType_ToString(type_0),
            name,
            path,
        );
    }
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Resource_Init() {
    Resource_AddPath(
        ResourceType_Font,
        b"../shared/res/font/%s.ttf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Font,
        b"../shared/res/font/%s.otf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"../shared/res/mesh/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"../shared/res/mesh/%s.obj\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Other,
        b"../shared/res/%s\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Script,
        b"../shared/res/script/%s.lua\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Shader,
        b"../shared/res/shader/%s.glsl\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"../shared/res/sound/%s.mp3\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"../shared/res/sound/%s.ogg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"../shared/res/sound/%s.ogx\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"../shared/res/sound/%s.wav\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex1D,
        b"../shared/res/tex1d/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"../shared/res/tex2d/%s.jpg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"../shared/res/tex2d/%s.png\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex3D,
        b"../shared/res/tex3d/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_TexCube,
        b"../shared/res/texcube/%s\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Font,
        b"./res/font/%s.ttf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Font,
        b"./res/font/%s.otf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"./res/mesh/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"./res/mesh/%s.obj\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Other,
        b"./res/%s\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Script,
        b"./res/script/%s.lua\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Shader,
        b"./res/shader/%s.glsl\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"./res/sound/%s.mp3\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"./res/sound/%s.ogg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"./res/sound/%s.ogx\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"./res/sound/%s.wav\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex1D,
        b"./res/tex1d/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"./res/tex2d/%s.jpg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"./res/tex2d/%s.png\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex3D,
        b"./res/tex3d/%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_TexCube,
        b"./res/texcube/%s\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(ResourceType_Font, b"%s.ttf\0" as *const u8 as *const libc::c_char);
    Resource_AddPath(ResourceType_Font, b"%s.otf\0" as *const u8 as *const libc::c_char);
    Resource_AddPath(ResourceType_Mesh, b"%s.bin\0" as *const u8 as *const libc::c_char);
    Resource_AddPath(ResourceType_Mesh, b"%s.obj\0" as *const u8 as *const libc::c_char);
    Resource_AddPath(ResourceType_Other, b"%s\0" as *const u8 as *const libc::c_char);
    Resource_AddPath(
        ResourceType_Script,
        b"%s.lua\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Shader,
        b"%s.glsl\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"%s.mp3\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"%s.ogg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"%s.ogx\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Sound,
        b"%s.wav\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex1D,
        b"%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"%s.jpg\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex2D,
        b"%s.png\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Tex3D,
        b"%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(ResourceType_TexCube, b"%s\0" as *const u8 as *const libc::c_char);
}
