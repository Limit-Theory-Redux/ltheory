use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use crate::Bytes::*;
use crate::TexFormat::*;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn glBegin(mode: GLenum);
    fn glBindTexture(target: GLenum, texture: GLu32);
    fn glDeleteTextures(n: GLsizei, textures: *const GLu32);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glEnd();
    fn glGenTextures(n: GLsizei, textures: *mut GLu32);
    fn glGetTexImage(
        target: GLenum,
        level: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut libc::c_void,
    );
    fn glTexCoord1f(s: GLfloat);
    fn glTexImage1D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        border: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *const libc::c_void,
    );
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    fn glTexSubImage1D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        width: GLsizei,
        format: GLenum,
        type_0: GLenum,
        pixels: *const libc::c_void,
    );
    fn glVertex2f(x: GLfloat, y: GLfloat);
    static mut __glewActiveTexture: PFNGLACTIVETEXTUREPROC;
    static mut __glewGenerateMipmap: PFNGLGENERATEMIPMAPPROC;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex1D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: i32,
    pub format: TexFormat,
}
pub type TexFormat = i32;
pub type DataFormat = i32;
pub type PixelFormat = i32;
pub type TexFilter = i32;
pub type TexWrapMode = i32;
pub type GLenum = u32;
pub type GLu32 = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLfloat = f32;
pub type PFNGLACTIVETEXTUREPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;
#[inline]
unsafe extern "C" fn Tex1D_Init() {
    glTexParameteri(
        0xde0 as i32 as GLenum,
        0x2800 as i32 as GLenum,
        0x2600 as i32,
    );
    glTexParameteri(
        0xde0 as i32 as GLenum,
        0x2801 as i32 as GLenum,
        0x2600 as i32,
    );
    glTexParameteri(
        0xde0 as i32 as GLenum,
        0x2802 as i32 as GLenum,
        0x812f as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Create(mut size: i32, mut format: TexFormat) -> *mut Tex1D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex1D_Create: Invalid texture format requested\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut this: *mut Tex1D = MemAlloc(::core::mem::size_of::<Tex1D>() as usize) as *mut Tex1D;
    (*this)._refCount = 1 as i32 as u32;
    (*this).size = size;
    (*this).format = format;
    glGenTextures(1 as i32, &mut (*this).handle);
    __glewActiveTexture.expect("non-null function pointer")(0x84c0 as i32 as GLenum);
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexImage1D(
        0xde0 as i32 as GLenum,
        0 as i32,
        (*this).format,
        (*this).size,
        0 as i32,
        (if TexFormat_IsColor(format) as i32 != 0 {
            0x1903 as i32
        } else {
            0x1902 as i32
        }) as GLenum,
        0x1401 as i32 as GLenum,
        std::ptr::null(),
    );
    Tex1D_Init();
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Acquire(mut this: *mut Tex1D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Free(mut this: *mut Tex1D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0 as i32 as u32
    } {
        glDeleteTextures(1 as i32, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Draw(
    mut this: *mut Tex1D,
    mut x: f32,
    mut y: f32,
    mut xs: f32,
    mut ys: f32,
) {
    glEnable(0xde0 as i32 as GLenum);
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glBegin(0x7 as i32 as GLenum);
    glTexCoord1f(0 as i32 as GLfloat);
    glVertex2f(x, y);
    glVertex2f(x, y + ys);
    glTexCoord1f(1 as i32 as GLfloat);
    glVertex2f(x + xs, y + ys);
    glVertex2f(x + xs, y);
    glEnd();
    glDisable(0xde0 as i32 as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GenMipmap(mut this: *mut Tex1D) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    __glewGenerateMipmap.expect("non-null function pointer")(0xde0 as i32 as GLenum);
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetFormat(mut this: *mut Tex1D) -> TexFormat {
    return (*this).format;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetData(
    mut this: *mut Tex1D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glGetTexImage(
        0xde0 as i32 as GLenum,
        0 as i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetDataBytes(
    mut this: *mut Tex1D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = (*this).size * DataFormat_GetSize(df) * PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as u32);
    Tex1D_GetData(this, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetHandle(mut this: *mut Tex1D) -> u32 {
    return (*this).handle;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetSize(mut this: *mut Tex1D) -> u32 {
    return (*this).size as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetData(
    mut this: *mut Tex1D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexImage1D(
        0xde0 as i32 as GLenum,
        0 as i32,
        (*this).format,
        (*this).size,
        0 as i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetDataBytes(
    mut this: *mut Tex1D,
    mut data: *mut Bytes,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Tex1D_SetData(this, Bytes_GetData(data), pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetMagFilter(mut this: *mut Tex1D, mut filter: TexFilter) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexParameteri(0xde0 as i32 as GLenum, 0x2800 as i32 as GLenum, filter);
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetMinFilter(mut this: *mut Tex1D, mut filter: TexFilter) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexParameteri(0xde0 as i32 as GLenum, 0x2801 as i32 as GLenum, filter);
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetTexel(
    mut this: *mut Tex1D,
    mut x: i32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    let mut rgba: [f32; 4] = [r, g, b, a];
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexSubImage1D(
        0xde0 as i32 as GLenum,
        0 as i32,
        x,
        1 as i32,
        0x1908 as i32 as GLenum,
        0x1406 as i32 as GLenum,
        rgba.as_mut_ptr() as *const libc::c_void,
    );
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetWrapMode(mut this: *mut Tex1D, mut mode: TexWrapMode) {
    glBindTexture(0xde0 as i32 as GLenum, (*this).handle);
    glTexParameteri(0xde0 as i32 as GLenum, 0x2802 as i32 as GLenum, mode);
    glBindTexture(0xde0 as i32 as GLenum, 0 as i32 as GLu32);
}
