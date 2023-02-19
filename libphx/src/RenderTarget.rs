use ::libc;
use glam::Vec3;
use glam::{IVec2, IVec3};
use crate::internal::Memory::*;
use crate::TexFormat::*;

extern "C" {
    pub type Tex2D;
    pub type Tex3D;
    pub type TexCube;
    fn Metric_Inc(_: Metric);
    fn Fatal(_: cstr, _: ...);
    static mut __glewDrawBuffers: PFNGLDRAWBUFFERSPROC;
    static mut __glewBindFramebuffer: PFNGLBINDFRAMEBUFFERPROC;
    static mut __glewDeleteFramebuffers: PFNGLDELETEFRAMEBUFFERSPROC;
    static mut __glewFramebufferTexture2D: PFNGLFRAMEBUFFERTEXTURE2DPROC;
    static mut __glewFramebufferTexture3D: PFNGLFRAMEBUFFERTEXTURE3DPROC;
    static mut __glewGenFramebuffers: PFNGLGENFRAMEBUFFERSPROC;
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn Tex2D_GetFormat(_: *mut Tex2D) -> TexFormat;
    fn Tex2D_GetHandle(_: *mut Tex2D) -> uint;
    fn Tex2D_GetSizeLevel(_: *mut Tex2D, out: *mut IVec2, level: libc::c_int);
    fn Tex3D_GetHandle(_: *mut Tex3D) -> uint;
    fn Tex3D_GetSizeLevel(_: *mut Tex3D, out: *mut IVec3, level: libc::c_int);
    fn TexCube_GetHandle(_: *mut TexCube) -> uint;
    fn TexFormat_IsColor(_: TexFormat) -> bool;
    fn Viewport_Pop();
    fn Viewport_Push(
        x: libc::c_int,
        y: libc::c_int,
        sx: libc::c_int,
        sy: libc::c_int,
        isWindow: bool,
    );
}
pub type int32_t = libc::c_int;
pub type uint = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;


pub type CubeFace = int32;
pub type Metric = int32;
pub type TexFormat = int32;
pub type GLenum = libc::c_uint;
pub type GLuint = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type PFNGLDRAWBUFFERSPROC = Option::<
    unsafe extern "C" fn(GLsizei, *const GLenum) -> (),
>;
pub type PFNGLBINDFRAMEBUFFERPROC = Option::<unsafe extern "C" fn(GLenum, GLuint) -> ()>;
pub type PFNGLDELETEFRAMEBUFFERSPROC = Option::<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
>;
pub type PFNGLFRAMEBUFFERTEXTURE2DPROC = Option::<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint) -> (),
>;
pub type PFNGLFRAMEBUFFERTEXTURE3DPROC = Option::<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLuint, GLint, GLint) -> (),
>;
pub type PFNGLGENFRAMEBUFFERSPROC = Option::<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FBO {
    pub handle: uint,
    pub colorIndex: libc::c_int,
    pub sx: libc::c_int,
    pub sy: libc::c_int,
    pub depth: bool,
}

static mut fboIndex: libc::c_int = -(1 as libc::c_int);
static mut fboStack: [FBO; 16] = [FBO {
    handle: 0,
    colorIndex: 0,
    sx: 0,
    sy: 0,
    depth: false,
}; 16];
#[inline]
unsafe extern "C" fn GetActive() -> *mut FBO {
    return fboStack.as_mut_ptr().offset(fboIndex as isize);
}
#[inline]
unsafe extern "C" fn SetDrawBuffers(mut count: libc::c_int) {
    static mut bufs: [GLenum; 4] = [
        0x8ce0 as libc::c_int as GLenum,
        0x8ce1 as libc::c_int as GLenum,
        0x8ce2 as libc::c_int as GLenum,
        0x8ce3 as libc::c_int as GLenum,
    ];
    __glewDrawBuffers.expect("non-null function pointer")(count, bufs.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Push(mut sx: libc::c_int, mut sy: libc::c_int) {
    Profiler_Begin(
        (*::core::mem::transmute::<
            &[u8; 18],
            &[libc::c_char; 18],
        >(b"RenderTarget_Push\0"))
            .as_ptr(),
    );
    if fboIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderTarget_Push: Maximum stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    fboIndex += 1;
    let mut self_0: *mut FBO = GetActive();
    (*self_0).handle = 0 as libc::c_int as uint;
    (*self_0).colorIndex = 0 as libc::c_int;
    (*self_0).sx = sx;
    (*self_0).sy = sy;
    (*self_0).depth = 0 as libc::c_int != 0;
    Metric_Inc(0x7 as libc::c_int);
    __glewGenFramebuffers
        .expect("non-null function pointer")(1 as libc::c_int, &mut (*self_0).handle);
    __glewBindFramebuffer
        .expect(
            "non-null function pointer",
        )(0x8d40 as libc::c_int as GLenum, (*self_0).handle);
    Viewport_Push(0 as libc::c_int, 0 as libc::c_int, sx, sy, 0 as libc::c_int != 0);
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Pop() {
    Profiler_Begin(
        (*::core::mem::transmute::<
            &[u8; 17],
            &[libc::c_char; 17],
        >(b"RenderTarget_Pop\0"))
            .as_ptr(),
    );
    if fboIndex < 0 as libc::c_int {
        Fatal(
            b"RenderTarget_Pop: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        __glewFramebufferTexture2D
            .expect(
                "non-null function pointer",
            )(
            0x8d40 as libc::c_int as GLenum,
            (0x8ce0 as libc::c_int + i) as GLenum,
            0xde1 as libc::c_int as GLenum,
            0 as libc::c_int as GLuint,
            0 as libc::c_int,
        );
        i += 1;
    }
    __glewFramebufferTexture2D
        .expect(
            "non-null function pointer",
        )(
        0x8d40 as libc::c_int as GLenum,
        0x8d00 as libc::c_int as GLenum,
        0xde1 as libc::c_int as GLenum,
        0 as libc::c_int as GLuint,
        0 as libc::c_int,
    );
    __glewDeleteFramebuffers
        .expect(
            "non-null function pointer",
        )(
        1 as libc::c_int,
        &mut (*fboStack.as_mut_ptr().offset(fboIndex as isize)).handle,
    );
    fboIndex -= 1;
    Metric_Inc(0x7 as libc::c_int);
    if fboIndex >= 0 as libc::c_int {
        __glewBindFramebuffer
            .expect(
                "non-null function pointer",
            )(0x8d40 as libc::c_int as GLenum, (*GetActive()).handle);
    } else {
        __glewBindFramebuffer
            .expect(
                "non-null function pointer",
            )(0x8d40 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    }
    Viewport_Pop();
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2D(mut self_0: *mut Tex2D) {
    RenderTarget_BindTex2DLevel(self_0, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2DLevel(
    mut tex: *mut Tex2D,
    mut level: libc::c_int,
) {
    let mut self_0: *mut FBO = GetActive();
    let mut handle: uint = Tex2D_GetHandle(tex);
    if TexFormat_IsColor(Tex2D_GetFormat(tex)) {
        if (*self_0).colorIndex >= 4 as libc::c_int {
            Fatal(
                b"RenderTarget_BindTex2D: Max color attachments exceeded\0" as *const u8
                    as *const libc::c_char,
            );
        }
        let fresh0 = (*self_0).colorIndex;
        (*self_0).colorIndex = (*self_0).colorIndex + 1;
        __glewFramebufferTexture2D
            .expect(
                "non-null function pointer",
            )(
            0x8d40 as libc::c_int as GLenum,
            (0x8ce0 as libc::c_int + fresh0) as GLenum,
            0xde1 as libc::c_int as GLenum,
            handle,
            level,
        );
        SetDrawBuffers((*self_0).colorIndex);
    } else {
        if (*self_0).depth {
            Fatal(
                b"RenderTarget_BindTex2D: Target already has a depth buffer\0"
                    as *const u8 as *const libc::c_char,
            );
        }
        __glewFramebufferTexture2D
            .expect(
                "non-null function pointer",
            )(
            0x8d40 as libc::c_int as GLenum,
            0x8d00 as libc::c_int as GLenum,
            0xde1 as libc::c_int as GLenum,
            handle,
            level,
        );
        (*self_0).depth = 1 as libc::c_int != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3D(
    mut self_0: *mut Tex3D,
    mut layer: libc::c_int,
) {
    RenderTarget_BindTex3DLevel(self_0, layer, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3DLevel(
    mut tex: *mut Tex3D,
    mut layer: libc::c_int,
    mut level: libc::c_int,
) {
    let mut self_0: *mut FBO = GetActive();
    if (*self_0).colorIndex >= 4 as libc::c_int {
        Fatal(
            b"RenderTarget_BindTex3D: Max color attachments exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut handle: uint = Tex3D_GetHandle(tex);
    let fresh1 = (*self_0).colorIndex;
    (*self_0).colorIndex = (*self_0).colorIndex + 1;
    __glewFramebufferTexture3D
        .expect(
            "non-null function pointer",
        )(
        0x8d40 as libc::c_int as GLenum,
        (0x8ce0 as libc::c_int + fresh1) as GLenum,
        0x806f as libc::c_int as GLenum,
        handle,
        level,
        layer,
    );
    SetDrawBuffers((*self_0).colorIndex);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCube(
    mut self_0: *mut TexCube,
    mut face: CubeFace,
) {
    RenderTarget_BindTexCubeLevel(self_0, face, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCubeLevel(
    mut tex: *mut TexCube,
    mut face: CubeFace,
    mut level: libc::c_int,
) {
    let mut self_0: *mut FBO = GetActive();
    if (*self_0).colorIndex >= 4 as libc::c_int {
        Fatal(
            b"RenderTarget_BindTexCubeLevel: Max color attachments exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut handle: uint = TexCube_GetHandle(tex);
    let fresh2 = (*self_0).colorIndex;
    (*self_0).colorIndex = (*self_0).colorIndex + 1;
    __glewFramebufferTexture2D
        .expect(
            "non-null function pointer",
        )(
        0x8d40 as libc::c_int as GLenum,
        (0x8ce0 as libc::c_int + fresh2) as GLenum,
        face as GLenum,
        handle,
        level,
    );
    SetDrawBuffers((*self_0).colorIndex);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2D(mut self_0: *mut Tex2D) {
    RenderTarget_PushTex2DLevel(self_0, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2DLevel(
    mut self_0: *mut Tex2D,
    mut level: libc::c_int,
) {
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Tex2D_GetSizeLevel(self_0, &mut size, level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex2DLevel(self_0, level);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3D(
    mut self_0: *mut Tex3D,
    mut layer: libc::c_int,
) {
    RenderTarget_PushTex3DLevel(self_0, layer, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3DLevel(
    mut self_0: *mut Tex3D,
    mut layer: libc::c_int,
    mut level: libc::c_int,
) {
    let mut size: IVec3 = IVec3 { x: 0, y: 0, z: 0 };
    Tex3D_GetSizeLevel(self_0, &mut size, level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex3DLevel(self_0, layer, level);
}
