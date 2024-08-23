use super::*;
use crate::common::*;
use crate::math::IVec3;
use crate::system::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FBO {
    pub handle: u32,
    pub colorIndex: i32,
    pub sx: i32,
    pub sy: i32,
    pub depth: bool,
}

static mut FBO_INDEX: i32 = -1;

static mut FBO_STACK: [FBO; 16] = [FBO {
    handle: 0,
    colorIndex: 0,
    sx: 0,
    sy: 0,
    depth: false,
}; 16];

#[inline]
unsafe extern "C" fn GetActive() -> *mut FBO {
    FBO_STACK.as_mut_ptr().offset(FBO_INDEX as isize)
}

#[inline]
unsafe extern "C" fn SetDrawBuffers(count: i32) {
    static mut BUFS: [gl::types::GLenum; 4] = [
        gl::COLOR_ATTACHMENT0 as _,
        gl::COLOR_ATTACHMENT1 as _,
        gl::COLOR_ATTACHMENT2 as _,
        gl::COLOR_ATTACHMENT3 as _,
    ];

    glcheck!(gl::DrawBuffers(count, BUFS.as_ptr()));
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Push(sx: i32, sy: i32) {
    Profiler_Begin(c_str!("RenderTarget_Push"));

    if FBO_INDEX + 1 >= 16 {
        panic!("RenderTarget_Push: Maximum stack depth exceeded");
    }

    FBO_INDEX += 1;

    let this: *mut FBO = GetActive();
    (*this).handle = 0;
    (*this).colorIndex = 0;
    (*this).sx = sx;
    (*this).sy = sy;
    (*this).depth = false;

    Metric_Inc(0x7);

    glcheck!(gl::GenFramebuffers(1, &mut (*this).handle));
    glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, (*this).handle));

    Viewport_Push(0, 0, sx, sy, false);

    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_Pop() {
    Profiler_Begin(c_str!("RenderTarget_Pop"));

    if FBO_INDEX < 0 {
        panic!("RenderTarget_Pop: Attempting to pop an empty stack");
    }

    let mut i: u32 = 0;
    while i < 4 {
        glcheck!(gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0 + i,
            gl::TEXTURE_2D,
            0,
            0,
        ));
        i += 1;
    }

    glcheck!(gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::DEPTH_ATTACHMENT,
        gl::TEXTURE_2D,
        0,
        0
    ));
    glcheck!(gl::DeleteFramebuffers(
        1,
        &(*FBO_STACK.as_mut_ptr().offset(FBO_INDEX as isize)).handle,
    ));

    FBO_INDEX -= 1;

    Metric_Inc(0x7);

    if FBO_INDEX >= 0 {
        glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, (*GetActive()).handle));
    } else {
        glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
    }

    Viewport_Pop();

    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2D(this: &Tex2D) {
    RenderTarget_BindTex2DLevel(this, 0);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex2DLevel(tex: &Tex2D, level: i32) {
    let this: *mut FBO = GetActive();
    let handle: u32 = Tex2D_GetHandle(tex);

    if TexFormat_IsColor(Tex2D_GetFormat(tex)) {
        if (*this).colorIndex >= 4 {
            panic!("RenderTarget_BindTex2D: Max color attachments exceeded");
        }

        glcheck!(gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0 + (*this).colorIndex as u32,
            gl::TEXTURE_2D,
            handle,
            level,
        ));
        (*this).colorIndex += 1;
        SetDrawBuffers((*this).colorIndex);
    } else {
        if (*this).depth {
            panic!("RenderTarget_BindTex2D: Target already has a depth buffer");
        }

        glcheck!(gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::TEXTURE_2D,
            handle,
            level,
        ));
        (*this).depth = true;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3D(this: &mut Tex3D, layer: i32) {
    RenderTarget_BindTex3DLevel(this, layer, 0);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTex3DLevel(tex: &mut Tex3D, layer: i32, level: i32) {
    let this: *mut FBO = GetActive();
    if (*this).colorIndex >= 4 {
        panic!("RenderTarget_BindTex3D: Max color attachments exceeded");
    }

    let handle: u32 = Tex3D_GetHandle(tex);
    glcheck!(gl::FramebufferTexture3D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0 + (*this).colorIndex as u32,
        gl::TEXTURE_3D,
        handle,
        level,
        layer,
    ));
    (*this).colorIndex += 1;
    SetDrawBuffers((*this).colorIndex);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCube(this: &mut TexCube, face: CubeFace) {
    RenderTarget_BindTexCubeLevel(this, face, 0);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_BindTexCubeLevel(
    tex: &mut TexCube,
    face: CubeFace,
    level: i32,
) {
    let this: *mut FBO = GetActive();
    if (*this).colorIndex >= 4 {
        panic!("RenderTarget_BindTexCubeLevel: Max color attachments exceeded");
    }
    let handle: u32 = TexCube_GetHandle(tex);

    glcheck!(gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0 + (*this).colorIndex as u32,
        face as u32,
        handle,
        level,
    ));
    (*this).colorIndex += 1;
    SetDrawBuffers((*this).colorIndex);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2D(this: &Tex2D) {
    RenderTarget_PushTex2DLevel(this, 0);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex2DLevel(this: &Tex2D, level: i32) {
    let size = this.get_size_level(level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex2DLevel(this, level);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3D(this: &mut Tex3D, layer: i32) {
    RenderTarget_PushTex3DLevel(this, layer, 0);
}

#[no_mangle]
pub unsafe extern "C" fn RenderTarget_PushTex3DLevel(this: &mut Tex3D, layer: i32, level: i32) {
    let mut size: IVec3 = IVec3 { x: 0, y: 0, z: 0 };
    Tex3D_GetSizeLevel(this, &mut size, level);
    RenderTarget_Push(size.x, size.y);
    RenderTarget_BindTex3DLevel(this, layer, level);
}
