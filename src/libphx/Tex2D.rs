use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Bytes;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Bytes_Rewind(_: *mut Bytes);
    fn DataFormat_GetSize(_: DataFormat) -> libc::c_int;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn Draw_Clear(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Metric_Inc(_: Metric);
    fn Metric_AddDrawImm(polys: int32, tris: int32, verts: int32);
    fn glBegin(mode: GLenum);
    fn glBindTexture(target: GLenum, texture: GLuint);
    fn glCopyTexImage2D(
        target: GLenum,
        level: GLint,
        internalFormat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
    );
    fn glDeleteTextures(n: GLsizei, textures: *const GLuint);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glEnd();
    fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    fn glGetTexImage(
        target: GLenum,
        level: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut libc::c_void,
    );
    fn glReadPixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut libc::c_void,
    );
    fn glTexCoord2f(s: GLfloat, t: GLfloat);
    fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *const libc::c_void,
    );
    fn glTexParameterf(target: GLenum, pname: GLenum, param: GLfloat);
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    fn glTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *const libc::c_void,
    );
    fn glVertex2f(x: GLfloat, y: GLfloat);
    static mut __glewActiveTexture: PFNGLACTIVETEXTUREPROC;
    static mut __glewGenerateMipmap: PFNGLGENERATEMIPMAPPROC;
    fn PixelFormat_Components(_: PixelFormat) -> libc::c_int;
    fn RenderTarget_Pop();
    fn RenderTarget_PushTex2D(_: *mut Tex2D);
    fn RenderTarget_PushTex2DLevel(_: *mut Tex2D, level: libc::c_int);
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
    fn TexFormat_IsColor(_: TexFormat) -> bool;
    fn TexFormat_IsValid(_: TexFormat) -> bool;
    fn Viewport_GetSize(out: *mut Vec2i);
    fn Tex2D_LoadRaw(
        path: cstr,
        sx: *mut libc::c_int,
        sy: *mut libc::c_int,
        components: *mut libc::c_int,
    ) -> *mut uchar;
    fn Tex2D_Save_Png(
        path: cstr,
        sx: libc::c_int,
        sy: libc::c_int,
        components: libc::c_int,
        data: *mut uchar,
    ) -> bool;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint = libc::c_uint;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex2D {
    pub _refCount: uint32,
    pub handle: uint,
    pub size: Vec2i,
    pub format: TexFormat,
}
pub type TexFormat = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
}
pub type DataFormat = int32;
pub type Metric = int32;
pub type PixelFormat = int32;
pub type ResourceType = int32;
pub type TexFilter = int32;
pub type TexWrapMode = int32;
pub type GLenum = libc::c_uint;
pub type GLuint = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type GLfloat = libc::c_float;
pub type PFNGLACTIVETEXTUREPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0;
#[inline]
unsafe extern "C" fn Vec2i_Create(mut x: libc::c_int, mut y: libc::c_int) -> Vec2i {
    let mut self_0: Vec2i = {
        let mut init = Vec2i { x: x, y: y };
        init
    };
    return self_0;
}

#[no_mangle]
pub static mut PixelFormat_Red: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RG: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGB: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGR: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGBA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGRA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_Depth_Component: PixelFormat = 0;
#[no_mangle]
pub static mut ResourceType_Font: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Mesh: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Other: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Script: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex3D: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_TexCube: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Shader: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Sound: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex1D: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex2D: ResourceType = 0;
#[no_mangle]
pub static mut TexFormat_R8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGB8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth24: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth32F: TexFormat = 0;
#[inline]
unsafe extern "C" fn Tex2D_Init() {
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2803 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Create(
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut format: TexFormat,
) -> *mut Tex2D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex2D_Create: Invalid texture format requested\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut self_0: *mut Tex2D = MemAlloc(
        ::core::mem::size_of::<Tex2D>() as libc::c_ulong,
    ) as *mut Tex2D;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    (*self_0).size = Vec2i_Create(sx, sy);
    (*self_0).format = format;
    glGenTextures(1 as libc::c_int, &mut (*self_0).handle);
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        (*self_0).format,
        (*self_0).size.x,
        (*self_0).size.y,
        0 as libc::c_int,
        (if TexFormat_IsColor(format) as libc::c_int != 0 {
            0x1903 as libc::c_int
        } else {
            0x1902 as libc::c_int
        }) as GLenum,
        0x1401 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    Tex2D_Init();
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_ScreenCapture() -> *mut Tex2D {
    let mut size: Vec2i = Vec2i { x: 0, y: 0 };
    Viewport_GetSize(&mut size);
    let mut self_0: *mut Tex2D = Tex2D_Create(size.x, size.y, TexFormat_RGBA8);
    let mut buf: *mut uint32 = MemAlloc(
        (::core::mem::size_of::<uint32>())
            .wrapping_mul((size.x * size.y) as libc::c_ulong),
    ) as *mut uint32;
    Metric_Inc(0x6 as libc::c_int);
    glReadPixels(
        0 as libc::c_int,
        0 as libc::c_int,
        size.x,
        size.y,
        0x1908 as libc::c_int as GLenum,
        0x1401 as libc::c_int as GLenum,
        buf as *mut libc::c_void,
    );
    let mut y: libc::c_int = 0 as libc::c_int;
    while y < size.y / 2 as libc::c_int {
        let mut x: libc::c_int = 0 as libc::c_int;
        while x < size.x {
            let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
            memcpy(
                swap_temp.as_mut_ptr() as *mut libc::c_void,
                &mut *buf.offset((size.x * (size.y - y - 1 as libc::c_int) + x) as isize)
                    as *mut uint32 as *const libc::c_void,
                ::core::mem::size_of::<uint32>() as libc::c_ulong,
            );
            memcpy(
                &mut *buf.offset((size.x * (size.y - y - 1 as libc::c_int) + x) as isize)
                    as *mut uint32 as *mut libc::c_void,
                &mut *buf.offset((size.x * y + x) as isize) as *mut uint32
                    as *const libc::c_void,
                ::core::mem::size_of::<uint32>() as libc::c_ulong,
            );
            memcpy(
                &mut *buf.offset((size.x * y + x) as isize) as *mut uint32
                    as *mut libc::c_void,
                swap_temp.as_mut_ptr() as *const libc::c_void,
                ::core::mem::size_of::<uint32>() as libc::c_ulong,
            );
            x += 1;
        }
        y += 1;
    }
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        TexFormat_RGBA8,
        size.x,
        size.y,
        0 as libc::c_int,
        0x1908 as libc::c_int as GLenum,
        0x1401 as libc::c_int as GLenum,
        buf as *const libc::c_void,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Acquire(mut self_0: *mut Tex2D) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Free(mut self_0: *mut Tex2D) {
    if !self_0.is_null()
        && {
            (*self_0)._refCount = ((*self_0)._refCount).wrapping_sub(1);
            (*self_0)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        glDeleteTextures(1 as libc::c_int, &mut (*self_0).handle);
        MemFree(self_0 as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Pop(_: *mut Tex2D) {
    RenderTarget_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Push(mut self_0: *mut Tex2D) {
    RenderTarget_PushTex2D(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_PushLevel(
    mut self_0: *mut Tex2D,
    mut level: libc::c_int,
) {
    RenderTarget_PushTex2DLevel(self_0, level);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clear(
    mut self_0: *mut Tex2D,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    RenderTarget_PushTex2D(self_0);
    Draw_Clear(r, g, b, a);
    RenderTarget_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clone(mut self_0: *mut Tex2D) -> *mut Tex2D {
    let mut clone: *mut Tex2D = Tex2D_Create(
        (*self_0).size.x,
        (*self_0).size.y,
        (*self_0).format,
    );
    RenderTarget_PushTex2D(self_0);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*clone).handle);
    glCopyTexImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        (*self_0).format as GLenum,
        0 as libc::c_int,
        0 as libc::c_int,
        (*self_0).size.x,
        (*self_0).size.y,
        0 as libc::c_int,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    RenderTarget_Pop();
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Draw(
    mut self_0: *mut Tex2D,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glEnable(0xde1 as libc::c_int as GLenum);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f(x, y);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f(x, y + sy);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f(x + sx, y + sy);
    glTexCoord2f(1 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f(x + sx, y);
    glEnd();
    glDisable(0xde1 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_DrawEx(
    mut self_0: *mut Tex2D,
    mut x0: libc::c_float,
    mut y0: libc::c_float,
    mut x1: libc::c_float,
    mut y1: libc::c_float,
    mut u0: libc::c_float,
    mut v0: libc::c_float,
    mut u1: libc::c_float,
    mut v1: libc::c_float,
) {
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glEnable(0xde1 as libc::c_int as GLenum);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord2f(u0, v0);
    glVertex2f(x0, y0);
    glTexCoord2f(u0, v1);
    glVertex2f(x0, y1);
    glTexCoord2f(u1, v1);
    glVertex2f(x1, y1);
    glTexCoord2f(u1, v0);
    glVertex2f(x1, y0);
    glEnd();
    glDisable(0xde1 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GenMipmap(mut self_0: *mut Tex2D) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0xde1 as libc::c_int as GLenum);
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetData(
    mut self_0: *mut Tex2D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Metric_Inc(0x6 as libc::c_int);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glGetTexImage(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetDataBytes(
    mut self_0: *mut Tex2D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: libc::c_int = (*self_0).size.x * (*self_0).size.y;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as uint32);
    Tex2D_GetData(self_0, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetFormat(mut self_0: *mut Tex2D) -> TexFormat {
    return (*self_0).format;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetHandle(mut self_0: *mut Tex2D) -> uint {
    return (*self_0).handle;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSize(mut self_0: *mut Tex2D, mut out: *mut Vec2i) {
    *out = (*self_0).size;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSizeLevel(
    mut self_0: *mut Tex2D,
    mut out: *mut Vec2i,
    mut level: libc::c_int,
) {
    *out = (*self_0).size;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < level {
        (*out).x /= 2 as libc::c_int;
        (*out).y /= 2 as libc::c_int;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Load(mut name: cstr) -> *mut Tex2D {
    let mut path: cstr = Resource_GetPath(ResourceType_Tex2D, name);
    let mut sx: libc::c_int = 0;
    let mut sy: libc::c_int = 0;
    let mut components: libc::c_int = 4 as libc::c_int;
    let mut data: *mut uchar = Tex2D_LoadRaw(path, &mut sx, &mut sy, &mut components);
    let mut self_0: *mut Tex2D = Tex2D_Create(sx, sy, TexFormat_RGBA8);
    let mut format: GLenum = (if components == 4 as libc::c_int {
        0x1908 as libc::c_int
    } else if components == 3 as libc::c_int {
        0x1907 as libc::c_int
    } else if components == 2 as libc::c_int {
        0x8227 as libc::c_int
    } else {
        0x1903 as libc::c_int
    }) as GLenum;
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        0x8058 as libc::c_int,
        (*self_0).size.x,
        (*self_0).size.y,
        0 as libc::c_int,
        format,
        0x1401 as libc::c_int as GLenum,
        data as *const libc::c_void,
    );
    Tex2D_Init();
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    MemFree(data as *const libc::c_void);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetAnisotropy(
    mut self_0: *mut Tex2D,
    mut factor: libc::c_float,
) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameterf(
        0xde1 as libc::c_int as GLenum,
        0x84fe as libc::c_int as GLenum,
        factor,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetData(
    mut self_0: *mut Tex2D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        (*self_0).format,
        (*self_0).size.x,
        (*self_0).size.y,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetDataBytes(
    mut self_0: *mut Tex2D,
    mut data: *mut Bytes,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Tex2D_SetData(self_0, Bytes_GetData(data), pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMagFilter(
    mut self_0: *mut Tex2D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMinFilter(
    mut self_0: *mut Tex2D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMipRange(
    mut self_0: *mut Tex2D,
    mut minLevel: libc::c_int,
    mut maxLevel: libc::c_int,
) {
    if minLevel != maxLevel {
        Warn(
            b"Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs.\0"
                as *const u8 as *const libc::c_char,
        );
    }
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x813c as libc::c_int as GLenum,
        minLevel,
    );
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x813d as libc::c_int as GLenum,
        maxLevel,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetTexel(
    mut self_0: *mut Tex2D,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut rgba: [libc::c_float; 4] = [r, g, b, a];
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexSubImage2D(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        x,
        y,
        1 as libc::c_int,
        1 as libc::c_int,
        0x1908 as libc::c_int as GLenum,
        0x1406 as libc::c_int as GLenum,
        rgba.as_mut_ptr() as *const libc::c_void,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetWrapMode(
    mut self_0: *mut Tex2D,
    mut mode: TexWrapMode,
) {
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        mode,
    );
    glTexParameteri(
        0xde1 as libc::c_int as GLenum,
        0x2803 as libc::c_int as GLenum,
        mode,
    );
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save(mut self_0: *mut Tex2D, mut path: cstr) {
    Metric_Inc(0x6 as libc::c_int);
    glBindTexture(0xde1 as libc::c_int as GLenum, (*self_0).handle);
    let mut buffer: *mut uchar = MemAlloc(
        (4 as libc::c_int * (*self_0).size.x * (*self_0).size.y) as size_t,
    ) as *mut uchar;
    glGetTexImage(
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int,
        0x1908 as libc::c_int as GLenum,
        0x1401 as libc::c_int as GLenum,
        buffer as *mut libc::c_void,
    );
    Tex2D_Save_Png(path, (*self_0).size.x, (*self_0).size.y, 4 as libc::c_int, buffer);
    MemFree(buffer as *const libc::c_void);
    glBindTexture(0xde1 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
