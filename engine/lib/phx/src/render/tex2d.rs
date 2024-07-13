use internal::*;

use super::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tex2D {
    pub _refCount: u32,
    pub handle: u32,
    pub size: IVec2,
    pub format: TexFormat,
}

#[inline]
extern "C" fn Tex2D_Init() {
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32
    ));
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

    glcheck!(gl::GenTextures(1, &mut (*this).handle));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, (*this).handle));
    glcheck!(gl::TexImage2D(
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
    ));

    Tex2D_Init();

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

    this
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_ScreenCapture() -> *mut Tex2D {
    let mut size: IVec2 = IVec2::ZERO;
    Viewport_GetSize(&mut size);

    let this = Tex2D_Create(size.x, size.y, TexFormat_RGBA8);
    let buf = MemNewArray!(u32, (size.x * size.y));

    Metric_Inc(0x6);

    glcheck!(gl::ReadPixels(
        0,
        0,
        size.x,
        size.y,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buf as *mut _,
    ));

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

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, (*this).handle));
    glcheck!(gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        TexFormat_RGBA8,
        size.x,
        size.y,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buf as *const _,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

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
        glcheck!(gl::DeleteTextures(1, &mut (*this).handle));
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

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, (*clone).handle));
    glcheck!(gl::CopyTexImage2D(
        gl::TEXTURE_2D,
        0,
        this.format as gl::types::GLenum,
        0,
        0,
        this.size.x,
        this.size.y,
        0,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

    RenderTarget_Pop();

    clone
}

#[no_mangle]
pub extern "C" fn Tex2D_GenMipmap(this: &mut Tex2D) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::GenerateMipmap(gl::TEXTURE_2D));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

#[no_mangle]
pub extern "C" fn Tex2D_GetData(
    this: &mut Tex2D,
    data: *mut libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    unsafe { Metric_Inc(0x6) };

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::GetTexImage(
        gl::TEXTURE_2D,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
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
    let path = Resource_GetPath(ResourceType::Tex2D, name);
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

    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, (*this).handle));
    glcheck!(gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA8 as i32,
        (*this).size.x,
        (*this).size.y,
        0,
        format,
        gl::UNSIGNED_BYTE,
        data as *const _,
    ));
    Tex2D_Init();
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

    MemFree(data as *const _);

    this
}

#[no_mangle]
pub extern "C" fn Tex2D_SetAnisotropy(this: &mut Tex2D, factor: f32) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexParameterf(
        gl::TEXTURE_2D,
        gl::TEXTURE_MAX_ANISOTROPY_EXT,
        factor
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

#[no_mangle]
pub extern "C" fn Tex2D_SetData(
    this: &mut Tex2D,
    data: *const libc::c_void,
    pf: PixelFormat,
    df: DataFormat,
) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        this.format,
        this.size.x,
        this.size.y,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
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
pub extern "C" fn Tex2D_SetMagFilter(this: &mut Tex2D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

#[no_mangle]
pub extern "C" fn Tex2D_SetMinFilter(this: &mut Tex2D, filter: TexFilter) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER,
        filter
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
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
pub extern "C" fn Tex2D_SetMipRange(this: &mut Tex2D, minLevel: i32, maxLevel: i32) {
    if minLevel != maxLevel {
        warn!("Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs.");
    }
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_BASE_LEVEL,
        minLevel
    ));
    glcheck!(gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MAX_LEVEL,
        maxLevel
    ));
}

#[no_mangle]
pub extern "C" fn Tex2D_SetTexel(this: &mut Tex2D, x: i32, y: i32, r: f32, g: f32, b: f32, a: f32) {
    let mut rgba: [f32; 4] = [r, g, b, a];

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexSubImage2D(
        gl::TEXTURE_2D,
        0,
        x,
        y,
        1,
        1,
        gl::RGBA,
        gl::FLOAT,
        rgba.as_mut_ptr() as *const _,
    ));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

#[no_mangle]
pub extern "C" fn Tex2D_SetWrapMode(this: &mut Tex2D, mode: TexWrapMode) {
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
    glcheck!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode));
    glcheck!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save(this: &mut Tex2D, path: *const libc::c_char) {
    Metric_Inc(0x6);

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));

    let buffer: *mut libc::c_uchar =
        MemAlloc((4 * this.size.x * this.size.y) as usize) as *mut libc::c_uchar;
    glcheck!(gl::GetTexImage(
        gl::TEXTURE_2D,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        buffer as *mut _,
    ));
    Tex2D_Save_Png(path, this.size.x, this.size.y, 4, buffer);
    MemFree(buffer as *const _);

    glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
}
