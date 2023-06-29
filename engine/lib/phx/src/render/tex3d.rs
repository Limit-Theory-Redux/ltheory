use super::*;
use crate::common::*;
use crate::internal::*;
use crate::math::*;
use crate::system::*;
use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex3D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: IVec3,
    pub format: TexFormat,
}

#[inline]
unsafe extern "C" fn Tex3D_Init() {
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
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
    gl::GenTextures(1, &mut (*this).handle);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::TexImage3D(
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
    );
    Tex3D_Init();
    gl::BindTexture(gl::TEXTURE_3D, 0);
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
        (*this)._refCount <= 0
    } {
        gl::DeleteTextures(1, &mut (*this).handle);
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
pub unsafe extern "C" fn Tex3D_Draw(
    this: &mut Tex3D,
    layer: i32,
    x: f32,
    y: f32,
    xs: f32,
    ys: f32,
) {
    let r: f32 = (layer + 1) as f32 / (this.size.z + 1) as f32;
    gl::Enable(gl::TEXTURE_3D);
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::Begin(gl::QUADS);
    gl::TexCoord3f(0.0f32, 0.0f32, r);
    gl::Vertex2f(x, y);
    gl::TexCoord3f(0.0f32, 1.0f32, r);
    gl::Vertex2f(x, y + ys);
    gl::TexCoord3f(1.0f32, 1.0f32, r);
    gl::Vertex2f(x + xs, y + ys);
    gl::TexCoord3f(1.0f32, 0.0f32, r);
    gl::Vertex2f(x + xs, y);
    gl::End();
    gl::Disable(gl::TEXTURE_3D);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GenMipmap(this: &mut Tex3D) {
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::GenerateMipmap(gl::TEXTURE_3D);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetData(
    this: &mut Tex3D,
    data: *mut libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::GetTexImage(
        gl::TEXTURE_3D,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetDataBytes(
    this: &mut Tex3D,
    pf: PixelFormat,
    df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = this.size.x * this.size.y * this.size.z;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let data: *mut Bytes = Bytes_Create(size as u32);
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
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::TexImage3D(
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
    );
    gl::BindTexture(gl::TEXTURE_3D, 0);
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
pub unsafe extern "C" fn Tex3D_SetMagFilter(this: &mut Tex3D, filter: TexFilter) {
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMinFilter(this: &mut Tex3D, filter: TexFilter) {
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetWrapMode(this: &mut Tex3D, mode: TexWrapMode) {
    gl::BindTexture(gl::TEXTURE_3D, this.handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, mode);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, mode);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, mode);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}
