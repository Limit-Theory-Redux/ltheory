use crate::internal::Memory::*;
use crate::Common::*;
use crate::Bytes::*;
use crate::DataFormat::*;
use crate::Math::Vec3;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use crate::TexFormat::*;
use crate::GL::gl;
use libc;

extern "C" {
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
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_Create(mut size: i32, mut format: TexFormat) -> *mut Tex1D {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"Tex1D_Create: Invalid texture format requested\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut this: *mut Tex1D = MemAlloc(::core::mem::size_of::<Tex1D>()) as *mut Tex1D;
    (*this)._refCount = 1_i32 as u32;
    (*this).size = size;
    (*this).format = format;
    gl::GenTextures(1_i32, &mut (*this).handle);
    __glewActiveTexture.expect("non-null function pointer")(gl::TEXTURE0);
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexImage1D(
        0xde0_i32 as GLenum,
        0_i32,
        (*this).format,
        (*this).size,
        0_i32,
        (if TexFormat_IsColor(format) as i32 != 0 {
            gl::RED
        } else {
            gl::DEPTH_COMPONENT
        }) as GLenum,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    Tex1D_Init();
    gl::BindTexture(0xde0_i32 as GLenum, 0);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_Acquire(mut this: *mut Tex1D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_Free(mut this: *mut Tex1D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        gl::DeleteTextures(1_i32, &mut (*this).handle);
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
    gl::Enable(0xde0_i32 as GLenum);
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::Begin(gl::QUADS);
    gl::TexCoord1f(0.0f32);
    gl::Vertex2f(x, y);
    gl::Vertex2f(x, y + ys);
    gl::TexCoord1f(1.0f32);
    gl::Vertex2f(x + xs, y + ys);
    gl::Vertex2f(x + xs, y);
    gl::End();
    gl::Disable(0xde0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GenMipmap(mut this: *mut Tex1D) {
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    __glewGenerateMipmap.expect("non-null function pointer")(0xde0_i32 as GLenum);
    gl::BindTexture(0xde0_i32 as GLenum, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetFormat(mut this: *mut Tex1D) -> TexFormat {
    (*this).format
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetData(
    mut this: *mut Tex1D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::GetTexImage(0xde0_i32 as GLenum, 0_i32, pf as GLenum, df as GLenum, data);
    gl::BindTexture(0xde0_i32 as GLenum, 0);
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
    data
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetHandle(mut this: *mut Tex1D) -> u32 {
    (*this).handle
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetSize(mut this: *mut Tex1D) -> u32 {
    (*this).size as u32
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetData(
    mut this: *mut Tex1D,
    mut data: *const libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexImage1D(
        0xde0_i32 as GLenum,
        0_i32,
        (*this).format,
        (*this).size,
        0_i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    gl::BindTexture(0xde0_i32 as GLenum, 0);
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
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_MAG_FILTER, filter);
    gl::BindTexture(0xde0_i32 as GLenum, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetMinFilter(mut this: *mut Tex1D, mut filter: TexFilter) {
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_MIN_FILTER, filter);
    gl::BindTexture(0xde0_i32 as GLenum, 0);
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
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexSubImage1D(
        0xde0_i32 as GLenum,
        0_i32,
        x,
        1_i32,
        gl::RGBA,
        gl::FLOAT,
        rgba.as_mut_ptr() as *const libc::c_void,
    );
    gl::BindTexture(0xde0_i32 as GLenum, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetWrapMode(mut this: *mut Tex1D, mut mode: TexWrapMode) {
    gl::BindTexture(0xde0_i32 as GLenum, (*this).handle);
    gl::TexParameteri(0xde0_i32 as GLenum, gl::TEXTURE_WRAP_S, mode);
    gl::BindTexture(0xde0_i32 as GLenum, 0);
}
