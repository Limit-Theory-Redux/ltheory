use ::libc;
use glam::Vec3;
use glam::IVec3;
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
extern "C" {
    pub type Bytes;
    fn Bytes_Rewind(_: *mut Bytes);
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Fatal(_: cstr, _: ...);
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
    fn glTexCoord3f(s: GLfloat, t: GLfloat, r: GLfloat);
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    fn glVertex2f(x: GLfloat, y: GLfloat);
    static mut __glewTexImage3D: PFNGLTEXIMAGE3DPROC;
    static mut __glewActiveTexture: PFNGLACTIVETEXTUREPROC;
    static mut __glewGenerateMipmap: PFNGLGENERATEMIPMAPPROC;
    fn PixelFormat_Components(_: PixelFormat) -> libc::c_int;
    fn RenderTarget_Pop();
    fn RenderTarget_PushTex3D(_: *mut Tex3D, layer: libc::c_int);
    fn RenderTarget_PushTex3DLevel(
        _: *mut Tex3D,
        layer: libc::c_int,
        level: libc::c_int,
    );
    fn TexFormat_IsDepth(_: TexFormat) -> bool;
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
pub struct Tex3D {
    pub _refCount: uint32,
    pub handle: uint,
    pub size: IVec3,
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
pub type PFNGLTEXIMAGE3DPROC = Option::<
    unsafe extern "C" fn(
        GLenum,
        GLint,
        GLint,
        GLsizei,
        GLsizei,
        GLsizei,
        GLint,
        GLenum,
        GLenum,
        *const libc::c_void,
    ) -> (),
>;
pub type PFNGLACTIVETEXTUREPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;

#[inline]
unsafe extern "C" fn Tex3D_Init() {
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2803 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x8072 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_Create(
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut sz: libc::c_int,
    mut format: TexFormat,
) -> *mut Tex3D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex3D_Create: Invalid texture format requested\0" as *const u8
                as *const libc::c_char,
        );
    }
    if TexFormat_IsDepth(format) {
        Fatal(
            b"Tex3D_Create: Cannot create 3D texture with depth format\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut self_0: *mut Tex3D = MemAlloc(
        ::core::mem::size_of::<Tex3D>() as usize,
    ) as *mut Tex3D;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    (*self_0).size = IVec3::new(sx, sy, sz);
    (*self_0).format = format;
    glGenTextures(1 as libc::c_int, &mut (*self_0).handle);
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    __glewTexImage3D
        .expect(
            "non-null function pointer",
        )(
        0x806f as libc::c_int as GLenum,
        0 as libc::c_int,
        (*self_0).format,
        (*self_0).size.x,
        (*self_0).size.y,
        (*self_0).size.z,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1401 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    Tex3D_Init();
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_Acquire(mut self_0: *mut Tex3D) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_Free(mut self_0: *mut Tex3D) {
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
pub unsafe extern "C" fn Tex3D_Pop(mut self_0: *mut Tex3D) {
    RenderTarget_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_Push(mut self_0: *mut Tex3D, mut layer: libc::c_int) {
    RenderTarget_PushTex3D(self_0, layer);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_PushLevel(
    mut self_0: *mut Tex3D,
    mut layer: libc::c_int,
    mut level: libc::c_int,
) {
    RenderTarget_PushTex3DLevel(self_0, layer, level);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_Draw(
    mut self_0: *mut Tex3D,
    mut layer: libc::c_int,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut xs: libc::c_float,
    mut ys: libc::c_float,
) {
    let mut r: libc::c_float = (layer + 1 as libc::c_int) as libc::c_float
        / ((*self_0).size.z + 1 as libc::c_int) as libc::c_float;
    glEnable(0x806f as libc::c_int as GLenum);
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord3f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat, r);
    glVertex2f(x, y);
    glTexCoord3f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat, r);
    glVertex2f(x, y + ys);
    glTexCoord3f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat, r);
    glVertex2f(x + xs, y + ys);
    glTexCoord3f(1 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat, r);
    glVertex2f(x + xs, y);
    glEnd();
    glDisable(0x806f as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GenMipmap(mut self_0: *mut Tex3D) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0x806f as libc::c_int as GLenum);
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetData(
    mut self_0: *mut Tex3D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    glGetTexImage(
        0x806f as libc::c_int as GLenum,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetDataBytes(
    mut self_0: *mut Tex3D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: libc::c_int = (*self_0).size.x * (*self_0).size.y * (*self_0).size.z;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as uint32);
    Tex3D_GetData(self_0, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetFormat(mut self_0: *mut Tex3D) -> TexFormat {
    return (*self_0).format;
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetHandle(mut self_0: *mut Tex3D) -> uint {
    return (*self_0).handle;
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetSize(mut self_0: *mut Tex3D, mut out: *mut IVec3) {
    *out = (*self_0).size;
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetSizeLevel(
    mut self_0: *mut Tex3D,
    mut out: *mut IVec3,
    mut level: libc::c_int,
) {
    *out = (*self_0).size;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < level {
        (*out).x /= 2 as libc::c_int;
        (*out).y /= 2 as libc::c_int;
        (*out).z /= 2 as libc::c_int;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetData(
    mut self_0: *mut Tex3D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    __glewTexImage3D
        .expect(
            "non-null function pointer",
        )(
        0x806f as libc::c_int as GLenum,
        0 as libc::c_int,
        (*self_0).format,
        (*self_0).size.x,
        (*self_0).size.y,
        (*self_0).size.z,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetDataBytes(
    mut self_0: *mut Tex3D,
    mut data: *mut Bytes,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Tex3D_SetData(self_0, Bytes_GetData(data), pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMagFilter(
    mut self_0: *mut Tex3D,
    mut filter: TexFilter,
) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMinFilter(
    mut self_0: *mut Tex3D,
    mut filter: TexFilter,
) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetWrapMode(
    mut self_0: *mut Tex3D,
    mut mode: TexWrapMode,
) {
    glBindTexture(0x806f as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        mode,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x2803 as libc::c_int as GLenum,
        mode,
    );
    glTexParameteri(
        0x806f as libc::c_int as GLenum,
        0x8072 as libc::c_int as GLenum,
        mode,
    );
    glBindTexture(0x806f as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
