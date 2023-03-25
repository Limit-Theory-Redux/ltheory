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
            b"Resource_Resolve: Failed to find %s <%s>\0" as *const u8 as *const libc::c_char,
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
            b"Resource_LoadBytes: Failed to load %s <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
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
            b"Resource_LoadCstr: Failed to load %s <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
            ResourceType_ToString(type_0),
            name,
            path,
        );
    }
    data
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
    Resource_AddPath(
        ResourceType_Font,
        b"%s.ttf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Font,
        b"%s.otf\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"%s.bin\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Mesh,
        b"%s.obj\0" as *const u8 as *const libc::c_char,
    );
    Resource_AddPath(
        ResourceType_Other,
        b"%s\0" as *const u8 as *const libc::c_char,
    );
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
    Resource_AddPath(
        ResourceType_TexCube,
        b"%s\0" as *const u8 as *const libc::c_char,
    );
}
