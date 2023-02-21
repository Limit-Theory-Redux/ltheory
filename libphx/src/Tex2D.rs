use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::ResourceType::*;
use crate::TexFormat::*;

extern "C" {
    pub type Bytes;
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
    fn Bytes_Create(len: u32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Bytes_Rewind(_: *mut Bytes);
    fn DataFormat_GetSize(_: DataFormat) -> i32;
    fn Draw_Clear(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn Metric_Inc(_: Metric);
    fn Metric_AddDrawImm(polys: i32, tris: i32, verts: i32);
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
    fn PixelFormat_Components(_: PixelFormat) -> i32;
    fn RenderTarget_Pop();
    fn RenderTarget_PushTex2D(_: *mut Tex2D);
    fn RenderTarget_PushTex2DLevel(_: *mut Tex2D, level: i32);
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
    fn TexFormat_IsColor(_: TexFormat) -> bool;
    fn TexFormat_IsValid(_: TexFormat) -> bool;
    fn Viewport_GetSize(out: *mut IVec2);
    fn Tex2D_LoadRaw(
        path: cstr,
        sx: *mut i32,
        sy: *mut i32,
        components: *mut i32,
    ) -> *mut uchar;
    fn Tex2D_Save_Png(
        path: cstr,
        sx: i32,
        sy: i32,
        components: i32,
        data: *mut uchar,
    ) -> bool;
}
pub type uint = u32;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex2D {
    pub _refCount: u32,
    pub handle: uint,
    pub size: IVec2,
    pub format: TexFormat,
}
pub type TexFormat = i32;

pub type DataFormat = i32;
pub type Metric = i32;
pub type PixelFormat = i32;
pub type ResourceType = i32;
pub type TexFilter = i32;
pub type TexWrapMode = i32;
pub type GLenum = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLfloat = f32;
pub type PFNGLACTIVETEXTUREPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;

#[inline]
unsafe extern "C" fn Tex2D_Init() {
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2800 as i32 as GLenum,
        0x2600 as i32,
    );
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2801 as i32 as GLenum,
        0x2600 as i32,
    );
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2802 as i32 as GLenum,
        0x812f as i32,
    );
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2803 as i32 as GLenum,
        0x812f as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Create(
    mut sx: i32,
    mut sy: i32,
    mut format: TexFormat,
) -> *mut Tex2D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex2D_Create: Invalid texture format requested\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut this: *mut Tex2D = MemAlloc(
        ::core::mem::size_of::<Tex2D>() as usize,
    ) as *mut Tex2D;
    (*this)._refCount = 1 as i32 as u32;
    (*this).size = IVec2::new(sx, sy);
    (*this).format = format;
    glGenTextures(1 as i32, &mut (*this).handle);
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as i32 as GLenum);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        0 as i32,
        (if TexFormat_IsColor(format) as i32 != 0 {
            0x1903 as i32
        } else {
            0x1902 as i32
        }) as GLenum,
        0x1401 as i32 as GLenum,
        0 as *const libc::c_void,
    );
    Tex2D_Init();
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_ScreenCapture() -> *mut Tex2D {
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Viewport_GetSize(&mut size);
    let mut this: *mut Tex2D = Tex2D_Create(size.x, size.y, TexFormat_RGBA8);
    let mut buf: *mut u32 = MemAlloc(
        (::core::mem::size_of::<u32>())
            .wrapping_mul((size.x * size.y) as usize),
    ) as *mut u32;
    Metric_Inc(0x6 as i32);
    glReadPixels(
        0 as i32,
        0 as i32,
        size.x,
        size.y,
        0x1908 as i32 as GLenum,
        0x1401 as i32 as GLenum,
        buf as *mut libc::c_void,
    );
    let mut y: i32 = 0 as i32;
    while y < size.y / 2 as i32 {
        let mut x: i32 = 0 as i32;
        while x < size.x {
            let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
            MemCpy(
                swap_temp.as_mut_ptr() as *mut libc::c_void,
                &mut *buf.offset((size.x * (size.y - y - 1 as i32) + x) as isize)
                    as *mut u32 as *const libc::c_void,
                ::core::mem::size_of::<u32>() as usize,
            );
            MemCpy(
                &mut *buf.offset((size.x * (size.y - y - 1 as i32) + x) as isize)
                    as *mut u32 as *mut libc::c_void,
                &mut *buf.offset((size.x * y + x) as isize) as *mut u32
                    as *const libc::c_void,
                ::core::mem::size_of::<u32>() as usize,
            );
            MemCpy(
                &mut *buf.offset((size.x * y + x) as isize) as *mut u32
                    as *mut libc::c_void,
                swap_temp.as_mut_ptr() as *const libc::c_void,
                ::core::mem::size_of::<u32>() as usize,
            );
            x += 1;
        }
        y += 1;
    }
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        TexFormat_RGBA8,
        size.x,
        size.y,
        0 as i32,
        0x1908 as i32 as GLenum,
        0x1401 as i32 as GLenum,
        buf as *const libc::c_void,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Acquire(mut this: *mut Tex2D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Free(mut this: *mut Tex2D) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as i32 as u32
        }
    {
        glDeleteTextures(1 as i32, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Pop(_: *mut Tex2D) {
    RenderTarget_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Push(mut this: *mut Tex2D) {
    RenderTarget_PushTex2D(this);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_PushLevel(
    mut this: *mut Tex2D,
    mut level: i32,
) {
    RenderTarget_PushTex2DLevel(this, level);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clear(
    mut this: *mut Tex2D,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    RenderTarget_PushTex2D(this);
    Draw_Clear(r, g, b, a);
    RenderTarget_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clone(mut this: *mut Tex2D) -> *mut Tex2D {
    let mut clone: *mut Tex2D = Tex2D_Create(
        (*this).size.x,
        (*this).size.y,
        (*this).format,
    );
    RenderTarget_PushTex2D(this);
    glBindTexture(0xde1 as i32 as GLenum, (*clone).handle);
    glCopyTexImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        (*this).format as GLenum,
        0 as i32,
        0 as i32,
        (*this).size.x,
        (*this).size.y,
        0 as i32,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
    RenderTarget_Pop();
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Draw(
    mut this: *mut Tex2D,
    mut x: f32,
    mut y: f32,
    mut sx: f32,
    mut sy: f32,
) {
    Metric_AddDrawImm(1 as i32, 2 as i32, 4 as i32);
    glEnable(0xde1 as i32 as GLenum);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glBegin(0x7 as i32 as GLenum);
    glTexCoord2f(0 as i32 as GLfloat, 0 as i32 as GLfloat);
    glVertex2f(x, y);
    glTexCoord2f(0 as i32 as GLfloat, 1 as i32 as GLfloat);
    glVertex2f(x, y + sy);
    glTexCoord2f(1 as i32 as GLfloat, 1 as i32 as GLfloat);
    glVertex2f(x + sx, y + sy);
    glTexCoord2f(1 as i32 as GLfloat, 0 as i32 as GLfloat);
    glVertex2f(x + sx, y);
    glEnd();
    glDisable(0xde1 as i32 as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_DrawEx(
    mut this: *mut Tex2D,
    mut x0: f32,
    mut y0: f32,
    mut x1: f32,
    mut y1: f32,
    mut u0: f32,
    mut v0: f32,
    mut u1: f32,
    mut v1: f32,
) {
    Metric_AddDrawImm(1 as i32, 2 as i32, 4 as i32);
    glEnable(0xde1 as i32 as GLenum);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glBegin(0x7 as i32 as GLenum);
    glTexCoord2f(u0, v0);
    glVertex2f(x0, y0);
    glTexCoord2f(u0, v1);
    glVertex2f(x0, y1);
    glTexCoord2f(u1, v1);
    glVertex2f(x1, y1);
    glTexCoord2f(u1, v0);
    glVertex2f(x1, y0);
    glEnd();
    glDisable(0xde1 as i32 as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GenMipmap(mut this: *mut Tex2D) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0xde1 as i32 as GLenum);
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetData(
    mut this: *mut Tex2D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Metric_Inc(0x6 as i32);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glGetTexImage(
        0xde1 as i32 as GLenum,
        0 as i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetDataBytes(
    mut this: *mut Tex2D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = (*this).size.x * (*this).size.y;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as u32);
    Tex2D_GetData(this, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetFormat(mut this: *mut Tex2D) -> TexFormat {
    return (*this).format;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetHandle(mut this: *mut Tex2D) -> uint {
    return (*this).handle;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSize(mut this: *mut Tex2D, mut out: *mut IVec2) {
    *out = (*this).size;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSizeLevel(
    mut this: *mut Tex2D,
    mut out: *mut IVec2,
    mut level: i32,
) {
    *out = (*this).size;
    let mut i: i32 = 0 as i32;
    while i < level {
        (*out).x /= 2 as i32;
        (*out).y /= 2 as i32;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Load(mut name: cstr) -> *mut Tex2D {
    let mut path: cstr = Resource_GetPath(ResourceType_Tex2D, name);
    let mut sx: i32 = 0;
    let mut sy: i32 = 0;
    let mut components: i32 = 4 as i32;
    let mut data: *mut uchar = Tex2D_LoadRaw(path, &mut sx, &mut sy, &mut components);
    let mut this: *mut Tex2D = Tex2D_Create(sx, sy, TexFormat_RGBA8);
    let mut format: GLenum = (if components == 4 as i32 {
        0x1908 as i32
    } else if components == 3 as i32 {
        0x1907 as i32
    } else if components == 2 as i32 {
        0x8227 as i32
    } else {
        0x1903 as i32
    }) as GLenum;
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as i32 as GLenum);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        0x8058 as i32,
        (*this).size.x,
        (*this).size.y,
        0 as i32,
        format,
        0x1401 as i32 as GLenum,
        data as *const libc::c_void,
    );
    Tex2D_Init();
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
    MemFree(data as *const libc::c_void);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetAnisotropy(
    mut this: *mut Tex2D,
    mut factor: f32,
) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexParameterf(
        0xde1 as i32 as GLenum,
        0x84fe as i32 as GLenum,
        factor,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetData(
    mut this: *mut Tex2D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        0 as i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetDataBytes(
    mut this: *mut Tex2D,
    mut data: *mut Bytes,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Tex2D_SetData(this, Bytes_GetData(data), pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMagFilter(
    mut this: *mut Tex2D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2800 as i32 as GLenum,
        filter,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMinFilter(
    mut this: *mut Tex2D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2801 as i32 as GLenum,
        filter,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMipRange(
    mut this: *mut Tex2D,
    mut minLevel: i32,
    mut maxLevel: i32,
) {
    if minLevel != maxLevel {
        Warn(
            b"Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs.\0"
                as *const u8 as *const libc::c_char,
        );
    }
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x813c as i32 as GLenum,
        minLevel,
    );
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x813d as i32 as GLenum,
        maxLevel,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetTexel(
    mut this: *mut Tex2D,
    mut x: i32,
    mut y: i32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    let mut rgba: [f32; 4] = [r, g, b, a];
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexSubImage2D(
        0xde1 as i32 as GLenum,
        0 as i32,
        x,
        y,
        1 as i32,
        1 as i32,
        0x1908 as i32 as GLenum,
        0x1406 as i32 as GLenum,
        rgba.as_mut_ptr() as *const libc::c_void,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetWrapMode(
    mut this: *mut Tex2D,
    mut mode: TexWrapMode,
) {
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2802 as i32 as GLenum,
        mode,
    );
    glTexParameteri(
        0xde1 as i32 as GLenum,
        0x2803 as i32 as GLenum,
        mode,
    );
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save(mut this: *mut Tex2D, mut path: cstr) {
    Metric_Inc(0x6 as i32);
    glBindTexture(0xde1 as i32 as GLenum, (*this).handle);
    let mut buffer: *mut uchar = MemAlloc(
        (4 as i32 * (*this).size.x * (*this).size.y) as usize,
    ) as *mut uchar;
    glGetTexImage(
        0xde1 as i32 as GLenum,
        0 as i32,
        0x1908 as i32 as GLenum,
        0x1401 as i32 as GLenum,
        buffer as *mut libc::c_void,
    );
    Tex2D_Save_Png(path, (*this).size.x, (*this).size.y, 4 as i32, buffer);
    MemFree(buffer as *const libc::c_void);
    glBindTexture(0xde1 as i32 as GLenum, 0 as i32 as GLuint);
}
