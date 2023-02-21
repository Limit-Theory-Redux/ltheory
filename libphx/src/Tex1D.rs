use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
extern "C" {
    pub type Bytes;
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Bytes_Rewind(_: *mut Bytes);
    fn DataFormat_GetSize(_: DataFormat) -> libc::c_int;
    fn glBegin(mode: GLenum);
    fn glBindTexture(target: GLenum, texture: GLuint);
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
    fn PixelFormat_Components(_: PixelFormat) -> libc::c_int;
    fn TexFormat_IsColor(_: TexFormat) -> bool;
    fn TexFormat_IsValid(_: TexFormat) -> bool;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex1D {
    pub _refCount: uint32,
    pub handle: uint,
    pub size: libc::c_int,
    pub format: TexFormat,
}
pub type TexFormat = int32;
pub type DataFormat = int32;
pub type PixelFormat = int32;
pub type TexFilter = int32;
pub type TexWrapMode = int32;
pub type GLenum = libc::c_uint;
pub type GLuint = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type GLfloat = libc::c_float;
pub type PFNGLACTIVETEXTUREPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
#[inline]
unsafe extern "C" fn Tex1D_Init() {
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Create(
    mut size: libc::c_int,
    mut format: TexFormat,
) -> *mut Tex1D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex1D_Create: Invalid texture format requested\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut this: *mut Tex1D = MemAlloc(
        ::core::mem::size_of::<Tex1D>() as usize,
    ) as *mut Tex1D;
    (*this)._refCount = 1 as libc::c_int as uint32;
    (*this).size = size;
    (*this).format = format;
    glGenTextures(1 as libc::c_int, &mut (*this).handle);
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexImage1D(
        0xde0 as libc::c_int as GLenum,
        0 as libc::c_int,
        (*this).format,
        (*this).size,
        0 as libc::c_int,
        (if TexFormat_IsColor(format) as libc::c_int != 0 {
            0x1903 as libc::c_int
        } else {
            0x1902 as libc::c_int
        }) as GLenum,
        0x1401 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    Tex1D_Init();
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Acquire(mut this: *mut Tex1D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Free(mut this: *mut Tex1D) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        glDeleteTextures(1 as libc::c_int, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_Draw(
    mut this: *mut Tex1D,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut xs: libc::c_float,
    mut ys: libc::c_float,
) {
    glEnable(0xde0 as libc::c_int as GLenum);
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord1f(0 as libc::c_int as GLfloat);
    glVertex2f(x, y);
    glVertex2f(x, y + ys);
    glTexCoord1f(1 as libc::c_int as GLfloat);
    glVertex2f(x + xs, y + ys);
    glVertex2f(x + xs, y);
    glEnd();
    glDisable(0xde0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GenMipmap(mut this: *mut Tex1D) {
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0xde0 as libc::c_int as GLenum);
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
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
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glGetTexImage(
        0xde0 as libc::c_int as GLenum,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetDataBytes(
    mut this: *mut Tex1D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: libc::c_int = (*this).size * DataFormat_GetSize(df)
        * PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as uint32);
    Tex1D_GetData(this, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetHandle(mut this: *mut Tex1D) -> uint {
    return (*this).handle;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetSize(mut this: *mut Tex1D) -> uint {
    return (*this).size as uint;
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetData(
    mut this: *mut Tex1D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexImage1D(
        0xde0 as libc::c_int as GLenum,
        0 as libc::c_int,
        (*this).format,
        (*this).size,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
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
pub unsafe extern "C" fn Tex1D_SetMagFilter(
    mut this: *mut Tex1D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetMinFilter(
    mut this: *mut Tex1D,
    mut filter: TexFilter,
) {
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetTexel(
    mut this: *mut Tex1D,
    mut x: libc::c_int,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut rgba: [libc::c_float; 4] = [r, g, b, a];
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexSubImage1D(
        0xde0 as libc::c_int as GLenum,
        0 as libc::c_int,
        x,
        1 as libc::c_int,
        0x1908 as libc::c_int as GLenum,
        0x1406 as libc::c_int as GLenum,
        rgba.as_mut_ptr() as *const libc::c_void,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetWrapMode(
    mut this: *mut Tex1D,
    mut mode: TexWrapMode,
) {
    glBindTexture(0xde0 as libc::c_int as GLenum, (*this).handle);
    glTexParameteri(
        0xde0 as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        mode,
    );
    glBindTexture(0xde0 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
