use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Common::*;
use crate::DataFormat::*;
use crate::Math::IVec3;
use crate::Math::Vec3;
use crate::PixelFormat::*;
use crate::RenderTarget::*;
use crate::TexFilter::*;
use crate::TexFormat::*;
use crate::TexWrapMode::*;
use crate::GL::gl;
use libc;

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
    let mut this = MemNew!(Tex3D);
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
pub unsafe extern "C" fn Tex3D_Acquire(mut this: *mut Tex3D) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_Free(mut this: *mut Tex3D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        gl::DeleteTextures(1, &mut (*this).handle);
        MemFree(this as *const _);
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
    let mut r: f32 = (layer + 1) as f32 / ((*this).size.z + 1) as f32;
    gl::Enable(gl::TEXTURE_3D);
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
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
pub unsafe extern "C" fn Tex3D_GenMipmap(mut this: *mut Tex3D) {
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::GenerateMipmap(gl::TEXTURE_3D);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_GetData(
    mut this: *mut Tex3D,
    mut data: *mut libc::c_void,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
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
    let mut i: i32 = 0;
    while i < level {
        (*out).x /= 2;
        (*out).y /= 2;
        (*out).z /= 2;
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
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::TexImage3D(
        gl::TEXTURE_3D,
        0,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        (*this).size.z,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl::BindTexture(gl::TEXTURE_3D, 0);
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
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetMinFilter(mut this: *mut Tex3D, mut filter: TexFilter) {
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex3D_SetWrapMode(mut this: *mut Tex3D, mut mode: TexWrapMode) {
    gl::BindTexture(gl::TEXTURE_3D, (*this).handle);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, mode);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, mode);
    gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, mode);
    gl::BindTexture(gl::TEXTURE_3D, 0);
}
