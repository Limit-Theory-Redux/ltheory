use internal::*;

use super::*;
use crate::common::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;
use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex2D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: IVec2,
    pub format: TexFormat,
}

#[inline]
unsafe extern "C" fn Tex2D_Init() {
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Create(sx: i32, sy: i32, format: TexFormat) -> *mut Tex2D {
    if !TexFormat_IsValid(format) {
        panic!("Tex2D_Create: Invalid texture format requested");
    }
    let this = MemNew!(Tex2D);
    (*this)._refCount = 1;
    (*this).size = IVec2::new(sx, sy);
    (*this).format = format;
    gl::GenTextures(1, &mut (*this).handle);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, (*this).handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        (*this).format,
        (*this).size.x,
        (*this).size.y,
        0,
        if TexFormat_IsColor(format) {
            gl::RED
        } else {
            gl::DEPTH_COMPONENT
        },
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    Tex2D_Init();
    gl::BindTexture(gl::TEXTURE_2D, 0);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_ScreenCapture() -> *mut Tex2D {
    let mut size: IVec2 = IVec2::ZERO;
    Viewport_GetSize(&mut size);
    let this = Tex2D_Create(size.x, size.y, TexFormat_RGBA8);
    let buf = MemNewArray!(u32, (size.x * size.y));
    Metric_Inc(0x6);
    gl::ReadPixels(
        0,
        0,
        size.x,
        size.y,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buf as *mut _,
    );
    let mut y: i32 = 0;
    while y < size.y / 2 {
        let mut x: i32 = 0;
        while x < size.x {
            let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
            MemCpy(
                swap_temp.as_mut_ptr() as *mut _,
                &mut *buf.offset((size.x * (size.y - y - 1) + x) as isize) as *mut u32 as *const _,
                std::mem::size_of::<u32>(),
            );
            MemCpy(
                &mut *buf.offset((size.x * (size.y - y - 1) + x) as isize) as *mut u32 as *mut _,
                &mut *buf.offset((size.x * y + x) as isize) as *mut u32 as *const _,
                std::mem::size_of::<u32>(),
            );
            MemCpy(
                &mut *buf.offset((size.x * y + x) as isize) as *mut u32 as *mut _,
                swap_temp.as_mut_ptr() as *const _,
                std::mem::size_of::<u32>(),
            );
            x += 1;
        }
        y += 1;
    }
    gl::BindTexture(gl::TEXTURE_2D, (*this).handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        TexFormat_RGBA8,
        size.x,
        size.y,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buf as *const _,
    );
    gl::BindTexture(gl::TEXTURE_2D, 0);
    this
}

#[no_mangle]
pub extern "C" fn Tex2D_Acquire(this: &mut Tex2D) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Free(this: *mut Tex2D) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        gl::DeleteTextures(1, &mut (*this).handle);
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Pop(_: *mut Tex2D) {
    RenderTarget_Pop();
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Push(this: &mut Tex2D) {
    RenderTarget_PushTex2D(this);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_PushLevel(this: &mut Tex2D, level: i32) {
    RenderTarget_PushTex2DLevel(this, level);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clear(this: &mut Tex2D, r: f32, g: f32, b: f32, a: f32) {
    RenderTarget_PushTex2D(this);
    Draw_Clear(r, g, b, a);
    RenderTarget_Pop();
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Clone(this: &mut Tex2D) -> *mut Tex2D {
    let clone: *mut Tex2D = Tex2D_Create(this.size.x, this.size.y, this.format);
    RenderTarget_PushTex2D(this);
    gl::BindTexture(gl::TEXTURE_2D, (*clone).handle);
    gl::CopyTexImage2D(
        gl::TEXTURE_2D,
        0,
        this.format as gl::types::GLenum,
        0,
        0,
        this.size.x,
        this.size.y,
        0,
    );
    gl::BindTexture(gl::TEXTURE_2D, 0);
    RenderTarget_Pop();
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Draw(this: &mut Tex2D, x: f32, y: f32, sx: f32, sy: f32) {
    Metric_AddDrawImm(1, 2, 4);
    gl::Enable(gl::TEXTURE_2D);
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::Begin(gl::QUADS);
    gl::TexCoord2f(0.0f32, 0.0f32);
    gl::Vertex2f(x, y);
    gl::TexCoord2f(0.0f32, 1.0f32);
    gl::Vertex2f(x, y + sy);
    gl::TexCoord2f(1.0f32, 1.0f32);
    gl::Vertex2f(x + sx, y + sy);
    gl::TexCoord2f(1.0f32, 0.0f32);
    gl::Vertex2f(x + sx, y);
    gl::End();
    gl::Disable(gl::TEXTURE_2D);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_DrawEx(
    this: &mut Tex2D,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
) {
    Metric_AddDrawImm(1, 2, 4);
    gl::Enable(gl::TEXTURE_2D);
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::Begin(gl::QUADS);
    gl::TexCoord2f(u0, v0);
    gl::Vertex2f(x0, y0);
    gl::TexCoord2f(u0, v1);
    gl::Vertex2f(x0, y1);
    gl::TexCoord2f(u1, v1);
    gl::Vertex2f(x1, y1);
    gl::TexCoord2f(u1, v0);
    gl::Vertex2f(x1, y0);
    gl::End();
    gl::Disable(gl::TEXTURE_2D);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_GenMipmap(this: &mut Tex2D) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::GenerateMipmap(gl::TEXTURE_2D);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetData(
    this: &mut Tex2D,
    data: *mut libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    Metric_Inc(0x6);
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::GetTexImage(
        gl::TEXTURE_2D,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetDataBytes(
    this: &mut Tex2D,
    pf: PixelFormat,
    df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = this.size.x * this.size.y;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let data: *mut Bytes = Bytes_Create(size as u32);
    Tex2D_GetData(this, Bytes_GetData(&mut *data), pf, df);
    Bytes_Rewind(&mut *data);
    data
}

#[no_mangle]
pub extern "C" fn Tex2D_GetFormat(this: &mut Tex2D) -> TexFormat {
    this.format
}

#[no_mangle]
pub extern "C" fn Tex2D_GetHandle(this: &mut Tex2D) -> u32 {
    this.handle
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSize(this: &mut Tex2D, out: *mut IVec2) {
    *out = this.size;
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_GetSizeLevel(this: &mut Tex2D, out: *mut IVec2, level: i32) {
    *out = this.size;
    let mut i: i32 = 0;
    while i < level {
        (*out).x /= 2;
        (*out).y /= 2;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Load(name: *const libc::c_char) -> *mut Tex2D {
    let path: *const libc::c_char = Resource_GetPath(ResourceType_Tex2D, name);
    let mut sx: i32 = 0;
    let mut sy: i32 = 0;
    let mut components: i32 = 4;
    let data: *mut libc::c_uchar = Tex2D_LoadRaw(path, &mut sx, &mut sy, &mut components);
    let this: *mut Tex2D = Tex2D_Create(sx, sy, TexFormat_RGBA8);

    let format = if components == 4 {
        gl::RGBA
    } else if components == 3 {
        gl::RGB
    } else if components == 2 {
        gl::RG
    } else {
        gl::RED
    };

    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, (*this).handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA8 as i32,
        (*this).size.x,
        (*this).size.y,
        0,
        format,
        gl::UNSIGNED_BYTE,
        data as *const _,
    );
    Tex2D_Init();
    gl::BindTexture(gl::TEXTURE_2D, 0);

    MemFree(data as *const _);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetAnisotropy(this: &mut Tex2D, factor: f32) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetData(
    this: &mut Tex2D,
    data: *const libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        this.format,
        this.size.x,
        this.size.y,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetDataBytes(
    this: &mut Tex2D,
    data: &mut Bytes,
    pf: PixelFormat,
    df: DataFormat,
) {
    Tex2D_SetData(this, Bytes_GetData(data), pf, df);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMagFilter(this: &mut Tex2D, filter: TexFilter) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMinFilter(this: &mut Tex2D, filter: TexFilter) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

/* NOTE : In general, using BASE_LEVEL, MAX_LEVEL, and MIN/MAX_LOD params is
 *        dangerous due to known bugs in old Radeon & Intel drivers. See:
 *        (https://www.opengl.org/discussion_boards/showthread.php/
 *         166266-Using-GL_TEXTURE_BASE_LEVEL-with-a-comple-texture)
 *
 *        However, constraining the mip range to a single level (minLevel ==
 *        maxLevel) seems to be acceptable even on bad drivers. Thus, it is
 *        strongly advised to use this function only to constrain sampling to
 *        a single mip level. */
#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetMipRange(this: &mut Tex2D, minLevel: i32, maxLevel: i32) {
    if minLevel != maxLevel {
        warn!("Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs.");
    }
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, minLevel);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, maxLevel);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetTexel(
    this: &mut Tex2D,
    x: i32,
    y: i32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let mut rgba: [f32; 4] = [r, g, b, a];
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexSubImage2D(
        gl::TEXTURE_2D,
        0,
        x,
        y,
        1,
        1,
        gl::RGBA,
        gl::FLOAT,
        rgba.as_mut_ptr() as *const _,
    );
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_SetWrapMode(this: &mut Tex2D, mode: TexWrapMode) {
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save(this: &mut Tex2D, path: *const libc::c_char) {
    Metric_Inc(0x6);
    gl::BindTexture(gl::TEXTURE_2D, this.handle);
    let buffer: *mut libc::c_uchar =
        MemAlloc((4 * this.size.x * this.size.y) as usize) as *mut libc::c_uchar;
    gl::GetTexImage(
        gl::TEXTURE_2D,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buffer as *mut _,
    );
    Tex2D_Save_Png(path, this.size.x, this.size.y, 4, buffer);
    MemFree(buffer as *const _);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}
