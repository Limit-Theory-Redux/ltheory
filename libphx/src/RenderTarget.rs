use crate::internal::Memory::*;
use crate::Metric::*;
use crate::Profiler::*;
use crate::Tex2D::*;
use crate::Tex3D::*;
use crate::TexCube::*;
use crate::TexFormat::*;
use crate::TexFormat::*;
use crate::Viewport::*;
use glam::Vec3;
use glam::{IVec2, IVec3};
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    static mut __glewDrawBuffers: PFNGLDRAWBUFFERSPROC;
    static mut __glewBindFramebuffer: PFNGLBINDFRAMEBUFFERPROC;
    static mut __glewDeleteFramebuffers: PFNGLDELETEFRAMEBUFFERSPROC;
    static mut __glewFramebufferTexture2D: PFNGLFRAMEBUFFERTEXTURE2DPROC;
    static mut __glewFramebufferTexture3D: PFNGLFRAMEBUFFERTEXTURE3DPROC;
    static mut __glewGenFramebuffers: PFNGLGENFRAMEBUFFERSPROC;
}

pub type CubeFace = i32;
pub type Metric = i32;
pub type TexFormat = i32;
pub type GLenum = u32;
pub type GLu32 = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type PFNGLDRAWBUFFERSPROC = Option<unsafe extern "C" fn(GLsizei, *const GLenum) -> ()>;
pub type PFNGLBINDFRAMEBUFFERPROC = Option<unsafe extern "C" fn(GLenum, GLu32) -> ()>;
pub type PFNGLDELETEFRAMEBUFFERSPROC = Option<unsafe extern "C" fn(GLsizei, *const GLu32) -> ()>;
pub type PFNGLFRAMEBUFFERTEXTURE2DPROC =
    Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLu32, GLint) -> ()>;
pub type PFNGLFRAMEBUFFERTEXTURE3DPROC =
    Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLu32, GLint, GLint) -> ()>;
pub type PFNGLGENFRAMEBUFFERSPROC = Option<unsafe extern "C" fn(GLsizei, *mut GLu32) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FBO {
    pub handle: u32,
    pub colorIndex: i32,
    pub sx: i32,
    pub sy: i32,
    pub depth: bool,
}

static mut fboIndex: i32 = -1_i32;
static mut fboStack: [FBO; 16] = [FBO {
    handle: 0,
    colorIndex: 0,
    sx: 0,
    sy: 0,
    depth: false,
}; 16];

#[inline]
unsafe extern "C" fn GetActive() -> *mut FBO {
    fboStack.as_mut_ptr().offset(fboIndex as isize)
}

#[inline]
unsafe extern "C" fn SetDrawBuffers(mut count: i32) {
    static mut bufs: [GLenum; 4] = [
        0x8ce0_i32 as GLenum,
        0x8ce1_i32 as GLenum,
        0x8ce2_i32 as GLenum,
        0x8ce3_i32 as GLenum,
    ];
    __glewDrawBuffers.expect("non-null function pointer")(count, bufs.as_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Push(mut sx: i32, mut sy: i32) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 18], &[libc::c_char; 18]>(b"RenderTarget_Push\0"))
            .as_ptr(),
    );
    if fboIndex + 1_i32 >= 16_i32 {
        Fatal(
            b"RenderTarget_Push: Maximum stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    fboIndex += 1;
    let mut this: *mut FBO = GetActive();
    (*this).handle = 0_i32 as u32;
    (*this).colorIndex = 0_i32;
    (*this).sx = sx;
    (*this).sy = sy;
    (*this).depth = false;
    Metric_Inc(0x7_i32);
    __glewGenFramebuffers.expect("non-null function pointer")(1_i32, &mut (*this).handle);
    __glewBindFramebuffer.expect("non-null function pointer")(0x8d40_i32 as GLenum, (*this).handle);
    Viewport_Push(0_i32, 0_i32, sx, sy, false);
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Pop() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 17], &[libc::c_char; 17]>(b"RenderTarget_Pop\0")).as_ptr(),
    );
    if fboIndex < 0_i32 {
        Fatal(
            b"RenderTarget_Pop: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut i: i32 = 0_i32;
    while i < 4_i32 {
        __glewFramebufferTexture2D.expect("non-null function pointer")(
            0x8d40_i32 as GLenum,
            (0x8ce0_i32 + i) as GLenum,
            0xde1_i32 as GLenum,
            0_i32 as GLu32,
            0_i32,
        );
        i += 1;
    }
    __glewFramebufferTexture2D.expect("non-null function pointer")(
        0x8d40_i32 as GLenum,
        0x8d00_i32 as GLenum,
        0xde1_i32 as GLenum,
        0_i32 as GLu32,
        0_i32,
    );
    __glewDeleteFramebuffers.expect("non-null function pointer")(
        1_i32,
        &mut (*fboStack.as_mut_ptr().offset(fboIndex as isize)).handle,
    );
    fboIndex -= 1;
    Metric_Inc(0x7_i32);
    if fboIndex >= 0_i32 {
        __glewBindFramebuffer.expect("non-null function pointer")(
            0x8d40_i32 as GLenum,
            (*GetActive()).handle,
        );
    } else {
        __glewBindFramebuffer.expect("non-null function pointer")(
            0x8d40_i32 as GLenum,
            0_i32 as GLu32,
        );
    }
    Viewport_Pop();
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2D(mut this: *mut Tex2D) {
    RenderTarget_BindTex2DLevel(this, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2DLevel(mut tex: *mut Tex2D, mut level: i32) {
    let mut this: *mut FBO = GetActive();
    let mut handle: u32 = Tex2D_GetHandle(tex);
    if TexFormat_IsColor(Tex2D_GetFormat(tex)) {
        if (*this).colorIndex >= 4_i32 {
            Fatal(
                b"RenderTarget_BindTex2D: Max color attachments exceeded\0" as *const u8
                    as *const libc::c_char,
            );
        }
        let fresh0 = (*this).colorIndex;
        (*this).colorIndex += 1;
        __glewFramebufferTexture2D.expect("non-null function pointer")(
            0x8d40_i32 as GLenum,
            (0x8ce0_i32 + fresh0) as GLenum,
            0xde1_i32 as GLenum,
            handle,
            level,
        );
        SetDrawBuffers((*this).colorIndex);
    } else {
        if (*this).depth {
            Fatal(
                b"RenderTarget_BindTex2D: Target already has a depth buffer\0" as *const u8
                    as *const libc::c_char,
            );
        }
        __glewFramebufferTexture2D.expect("non-null function pointer")(
            0x8d40_i32 as GLenum,
            0x8d00_i32 as GLenum,
            0xde1_i32 as GLenum,
            handle,
            level,
        );
        (*this).depth = true;
    };
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3D(mut this: *mut Tex3D, mut layer: i32) {
    RenderTarget_BindTex3DLevel(this, layer, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3DLevel(
    mut tex: *mut Tex3D,
    mut layer: i32,
    mut level: i32,
) {
    let mut this: *mut FBO = GetActive();
    if (*this).colorIndex >= 4_i32 {
        Fatal(
            b"RenderTarget_BindTex3D: Max color attachments exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut handle: u32 = Tex3D_GetHandle(tex);
    let fresh1 = (*this).colorIndex;
    (*this).colorIndex += 1;
    __glewFramebufferTexture3D.expect("non-null function pointer")(
        0x8d40_i32 as GLenum,
        (0x8ce0_i32 + fresh1) as GLenum,
        0x806f_i32 as GLenum,
        handle,
        level,
        layer,
    );
    SetDrawBuffers((*this).colorIndex);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCube(mut this: *mut TexCube, mut face: CubeFace) {
    RenderTarget_BindTexCubeLevel(this, face, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCubeLevel(
    mut tex: *mut TexCube,
    mut face: CubeFace,
    mut level: i32,
) {
    let mut this: *mut FBO = GetActive();
    if (*this).colorIndex >= 4_i32 {
        Fatal(
            b"RenderTarget_BindTexCubeLevel: Max color attachments exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut handle: u32 = TexCube_GetHandle(tex);
    let fresh2 = (*this).colorIndex;
    (*this).colorIndex += 1;
    __glewFramebufferTexture2D.expect("non-null function pointer")(
        0x8d40_i32 as GLenum,
        (0x8ce0_i32 + fresh2) as GLenum,
        face as GLenum,
        handle,
        level,
    );
    SetDrawBuffers((*this).colorIndex);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2D(mut this: *mut Tex2D) {
    RenderTarget_PushTex2DLevel(this, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2DLevel(mut this: *mut Tex2D, mut level: i32) {
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Tex2D_GetSizeLevel(this, &mut size, level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex2DLevel(this, level);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3D(mut this: *mut Tex3D, mut layer: i32) {
    RenderTarget_PushTex3DLevel(this, layer, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3DLevel(
    mut this: *mut Tex3D,
    mut layer: i32,
    mut level: i32,
) {
    let mut size: IVec3 = IVec3 { x: 0, y: 0, z: 0 };
    Tex3D_GetSizeLevel(this, &mut size, level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex3DLevel(this, layer, level);
}
