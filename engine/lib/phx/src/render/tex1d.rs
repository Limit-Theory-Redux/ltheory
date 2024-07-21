use internal::*;

use super::*;
use crate::system::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex1D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: i32,
    pub format: TexFormat,
}

#[inline]
extern "C" fn Tex1D_Init() {
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_1D,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_1D,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_1D,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_Create(size: i32, format: TexFormat) -> *mut Tex1D {
    if !TexFormat_IsValid(format) {
        panic!("Tex1D_Create: Invalid texture format requested");
    }

    let this = MemNew!(Tex1D);
    (*this)._refCount = 1;
    (*this).size = size;
    (*this).format = format;

    glcheck!(gl::GenTextures(1, &mut (*this).handle));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, (*this).handle));
    glcheck!(gl::TexImage1D(
        gl::TEXTURE_1D,
        0,
        (*this).format,
        (*this).size,
        0,
        (if TexFormat_IsColor(format) as i32 != 0 {
            gl::RED
        } else {
            gl::DEPTH_COMPONENT
        }) as gl::types::GLenum,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    ));

    Tex1D_Init();

    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));

    this
}

#[no_mangle]
pub extern "C" fn Tex1D_Acquire(this: &mut Tex1D) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_Free(this: *mut Tex1D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount == 0
    } {
        glcheck!(gl::DeleteTextures(1, &(*this).handle));
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub extern "C" fn Tex1D_GenMipmap(this: &mut Tex1D) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::GenerateMipmap(gl::TEXTURE_1D));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub extern "C" fn Tex1D_GetFormat(this: &mut Tex1D) -> TexFormat {
    this.format
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetData(
    this: &mut Tex1D,
    data: *mut libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::GetTexImage(
        gl::TEXTURE_1D,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_GetDataBytes(
    this: &mut Tex1D,
    pf: PixelFormat,
    df: DataFormat,
) -> *mut Bytes {
    let size: i32 = this.size * DataFormat_GetSize(df) * PixelFormat_Components(pf);
    let data: *mut Bytes = Bytes_Create(size as u32);
    Tex1D_GetData(this, Bytes_GetData(&mut *data), pf, df);
    Bytes_Rewind(&mut *data);
    data
}

#[no_mangle]
pub extern "C" fn Tex1D_GetHandle(this: &mut Tex1D) -> u32 {
    this.handle
}

#[no_mangle]
pub extern "C" fn Tex1D_GetSize(this: &mut Tex1D) -> u32 {
    this.size as u32
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetData(
    this: &mut Tex1D,
    data: *const libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::TexImage1D(
        gl::TEXTURE_1D,
        0,
        this.format,
        this.size,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex1D_SetDataBytes(
    this: &mut Tex1D,
    data: *mut Bytes,
    pf: PixelFormat,
    df: DataFormat,
) {
    Tex1D_SetData(this, Bytes_GetData(&mut *data), pf, df);
}

#[no_mangle]
pub extern "C" fn Tex1D_SetMagFilter(this: &mut Tex1D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_1D,
        gl::TEXTURE_MAG_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub extern "C" fn Tex1D_SetMinFilter(this: &mut Tex1D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_1D,
        gl::TEXTURE_MIN_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub extern "C" fn Tex1D_SetTexel(this: &mut Tex1D, x: i32, r: f32, g: f32, b: f32, a: f32) {
    let mut rgba: [f32; 4] = [r, g, b, a];

    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::TexSubImage1D(
        gl::TEXTURE_1D,
        0,
        x,
        1,
        gl::RGBA,
        gl::FLOAT,
        rgba.as_mut_ptr() as *const _,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}

#[no_mangle]
pub extern "C" fn Tex1D_SetWrapMode(this: &mut Tex1D, mode: TexWrapMode) {
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
    glcheck!(gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_WRAP_S, mode));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
}
