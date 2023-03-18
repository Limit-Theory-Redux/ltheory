use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::RenderTarget::*;
use crate::TexFormat::*;
use crate::PixelFormat::*;
use crate::Bytes::*;
use glam::IVec3;
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
    fn glTexCoord3f(s: GLfloat, t: GLfloat, r: GLfloat);
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    fn glVertex2f(x: GLfloat, y: GLfloat);
    static mut __glewTexImage3D: PFNGLTEXIMAGE3DPROC;
    static mut __glewActiveTexture: PFNGLACTIVETEXTUREPROC;
    static mut __glewGenerateMipmap: PFNGLGENERATEMIPMAPPROC;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex3D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: IVec3,
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
pub type PFNGLTEXIMAGE3DPROC = Option<
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
pub type PFNGLACTIVETEXTUREPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLGENERATEMIPMAPPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;

#[inline]
unsafe extern "C" fn Tex3D_Init() {
    glTexParameteri(
        0x806f_i32 as GLenum,
        0x2800_i32 as GLenum,
        0x2600_i32,
    );
    glTexParameteri(
        0x806f_i32 as GLenum,
        0x2801_i32 as GLenum,
        0x2600_i32,
    );
    glTexParameteri(
        0x806f_i32 as GLenum,
        0x2802_i32 as GLenum,
        0x812f_i32,
    );
    glTexParameteri(
        0x806f_i32 as GLenum,
        0x2803_i32 as GLenum,
        0x812f_i32,
    );
    glTexParameteri(
        0x806f_i32 as GLenum,
        0x8072_i32 as GLenum,
        0x812f_i32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Create(
    mut sx: i32,
    mut sy: i32,
    mut sz: i32,
    mut format: TexFormat,
) -> *mut Tex3D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex3D_Create: Invalid texture format requested\0" as *const u8 as *const libc::c_char,
        );
    }
    if TexFormat_IsDepth(format) {
        Fatal(
            b"Tex3D_Create: Cannot create 3D texture with depth format\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut this: *mut Tex3D = MemAlloc(::core::mem::size_of::<Tex3D>()) as *mut Tex3D;
    (*this)._refCount = 1_i32 as u32;
    (*this).size = IVec3::new(sx, sy, sz);
    (*this).format = format;
    glGenTextures(1_i32, &mut (*this).handle);
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    __glewTexImage3D.expect("non-null function pointer")(
        0x806f_i32 as GLenum,
        0_i32,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        (*this).size.z,
        0_i32,
        0x1903_i32 as GLenum,
        0x1401_i32 as GLenum,
        std::ptr::null(),
    );
    Tex3D_Init();
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Acquire(mut this: *mut Tex3D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Free(mut this: *mut Tex3D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        glDeleteTextures(1_i32, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Pop(mut _this: *mut Tex3D) {
    RenderTarget_Pop();
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Push(mut this: *mut Tex3D, mut layer: i32) {
    RenderTarget_PushTex3D(this, layer);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_PushLevel(mut this: *mut Tex3D, mut layer: i32, mut level: i32) {
    RenderTarget_PushTex3DLevel(this, layer, level);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Draw(
    mut this: *mut Tex3D,
    mut layer: i32,
    mut x: f32,
    mut y: f32,
    mut xs: f32,
    mut ys: f32,
) {
    let mut r: f32 = (layer + 1_i32) as f32 / ((*this).size.z + 1_i32) as f32;
    glEnable(0x806f_i32 as GLenum);
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    glBegin(0x7_i32 as GLenum);
    glTexCoord3f(0_i32 as GLfloat, 0_i32 as GLfloat, r);
    glVertex2f(x, y);
    glTexCoord3f(0_i32 as GLfloat, 1_i32 as GLfloat, r);
    glVertex2f(x, y + ys);
    glTexCoord3f(1_i32 as GLfloat, 1_i32 as GLfloat, r);
    glVertex2f(x + xs, y + ys);
    glTexCoord3f(1_i32 as GLfloat, 0_i32 as GLfloat, r);
    glVertex2f(x + xs, y);
    glEnd();
    glDisable(0x806f_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GenMipmap(mut this: *mut Tex3D) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    __glewGenerateMipmap.expect("non-null function pointer")(0x806f_i32 as GLenum);
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetData(
    mut this: *mut Tex3D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    glGetTexImage(
        0x806f_i32 as GLenum,
        0_i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetDataBytes(
    mut this: *mut Tex3D,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = (*this).size.x * (*this).size.y * (*this).size.z;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as u32);
    Tex3D_GetData(this, Bytes_GetData(data), pf, df);
    Bytes_Rewind(data);
    data
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetFormat(mut this: *mut Tex3D) -> TexFormat {
    (*this).format
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetHandle(mut this: *mut Tex3D) -> u32 {
    (*this).handle
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetSize(mut this: *mut Tex3D, mut out: *mut IVec3) {
    *out = (*this).size;
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetSizeLevel(
    mut this: *mut Tex3D,
    mut out: *mut IVec3,
    mut level: i32,
) {
    *out = (*this).size;
    let mut i: i32 = 0_i32;
    while i < level {
        (*out).x /= 2_i32;
        (*out).y /= 2_i32;
        (*out).z /= 2_i32;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetData(
    mut this: *mut Tex3D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    __glewTexImage3D.expect("non-null function pointer")(
        0x806f_i32 as GLenum,
        0_i32,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        (*this).size.z,
        0_i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetDataBytes(
    mut this: *mut Tex3D,
    mut data: *mut Bytes,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    Tex3D_SetData(this, Bytes_GetData(data), pf, df);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMagFilter(mut this: *mut Tex3D, mut filter: TexFilter) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    glTexParameteri(0x806f_i32 as GLenum, 0x2800_i32 as GLenum, filter);
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMinFilter(mut this: *mut Tex3D, mut filter: TexFilter) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    glTexParameteri(0x806f_i32 as GLenum, 0x2801_i32 as GLenum, filter);
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetWrapMode(mut this: *mut Tex3D, mut mode: TexWrapMode) {
    glBindTexture(0x806f_i32 as GLenum, (*this).handle);
    glTexParameteri(0x806f_i32 as GLenum, 0x2802_i32 as GLenum, mode);
    glTexParameteri(0x806f_i32 as GLenum, 0x2803_i32 as GLenum, mode);
    glTexParameteri(0x806f_i32 as GLenum, 0x8072_i32 as GLenum, mode);
    glBindTexture(0x806f_i32 as GLenum, 0_i32 as GLu32);
}