use internal::*;

use super::*;
use crate::math::*;
use crate::system::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex3D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: IVec3,
    pub format: TexFormat,
}

#[inline]
extern "C" fn Tex3D_Init() {
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_WRAP_R,
        gl::CLAMP_TO_EDGE as i32
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Create(sx: i32, sy: i32, sz: i32, format: TexFormat) -> *mut Tex3D {
    if !TexFormat_IsValid(format) {
        panic!("Tex3D_Create: Invalid texture format requested");
    }

    if TexFormat_IsDepth(format) {
        panic!("Tex3D_Create: Cannot create 3D texture with depth format");
    }

    let this = MemNew!(Tex3D);
    (*this)._refCount = 1;
    (*this).size = IVec3::new(sx, sy, sz);
    (*this).format = format;

    glcheck!(gl::GenTextures(1, &mut (*this).handle));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, (*this).handle));
    glcheck!(gl::TexImage3D(
        gl::TEXTURE_3D,
        0,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        (*this).size.z,
        0,
        gl::RED,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    ));
    Tex3D_Init();
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));

    this
}

#[no_mangle]
pub extern "C" fn Tex3D_Acquire(this: &mut Tex3D) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Free(this: *mut Tex3D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount == 0
    } {
        glcheck!(gl::DeleteTextures(1, &(*this).handle));
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Pop(_this: &mut Tex3D) {
    RenderTarget_Pop();
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Push(this: &mut Tex3D, layer: i32) {
    RenderTarget_PushTex3D(this, layer);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_PushLevel(this: &mut Tex3D, layer: i32, level: i32) {
    RenderTarget_PushTex3DLevel(this, layer, level);
}

#[no_mangle]
pub extern "C" fn Tex3D_GenMipmap(this: &mut Tex3D) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::GenerateMipmap(gl::TEXTURE_3D));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetData(
    this: &mut Tex3D,
    data: *mut libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::GetTexImage(
        gl::TEXTURE_3D,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetDataBytes(
    this: &mut Tex3D,
    pf: PixelFormat,
    df: DataFormat,
) -> Box<Bytes> {
    let mut size: i32 = this.size.x * this.size.y * this.size.z;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);

    let mut data = Bytes_Create(size as u32);
    Tex3D_GetData(this, Bytes_GetData(&mut *data), pf, df);
    Bytes_Rewind(&mut *data);

    data
}

#[no_mangle]
pub extern "C" fn Tex3D_GetFormat(this: &mut Tex3D) -> TexFormat {
    this.format
}

#[no_mangle]
pub extern "C" fn Tex3D_GetHandle(this: &mut Tex3D) -> u32 {
    this.handle
}

#[no_mangle]
pub extern "C" fn Tex3D_GetSize(this: &mut Tex3D, out: &mut IVec3) {
    *out = this.size;
}

#[no_mangle]
pub extern "C" fn Tex3D_GetSizeLevel(this: &mut Tex3D, out: &mut IVec3, level: i32) {
    *out = this.size;
    let mut i: i32 = 0;
    while i < level {
        out.x /= 2;
        out.y /= 2;
        out.z /= 2;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetData(
    this: &mut Tex3D,
    data: *const libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::TexImage3D(
        gl::TEXTURE_3D,
        0,
        this.format,
        this.size.x,
        this.size.y,
        this.size.z,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetDataBytes(
    this: &mut Tex3D,
    data: &mut Bytes,
    pf: PixelFormat,
    df: DataFormat,
) {
    Tex3D_SetData(this, Bytes_GetData(data), pf, df);
}

#[no_mangle]
pub extern "C" fn Tex3D_SetMagFilter(this: &mut Tex3D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_MAG_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}

#[no_mangle]
pub extern "C" fn Tex3D_SetMinFilter(this: &mut Tex3D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_3D,
        gl::TEXTURE_MIN_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}

#[no_mangle]
pub extern "C" fn Tex3D_SetWrapMode(this: &mut Tex3D, mode: TexWrapMode) {
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
    glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, mode));
    glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, mode));
    glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, mode));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
}
