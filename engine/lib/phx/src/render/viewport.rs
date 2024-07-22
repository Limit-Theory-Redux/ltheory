use super::{gl, *};
use crate::math::*;

/* TODO : This is a low-level mechanism and probably not for use outside of
 *        RenderTarget. Should likely be folded into RenderTarget. */

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VP {
    pub x: i32,
    pub y: i32,
    pub sx: i32,
    pub sy: i32,
    pub isWindow: bool,
}
static mut VP_INDEX: i32 = -1;

static mut VP: [VP; 16] = [VP {
    x: 0,
    y: 0,
    sx: 0,
    sy: 0,
    isWindow: false,
}; 16];

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetAspect() -> f32 {
    if VP_INDEX < 0 {
        panic!("Viewport_GetAspect: Viewport stack is empty");
    }
    VP[VP_INDEX as usize].sx as f32 / VP[VP_INDEX as usize].sy as f32
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(out: &mut IVec2) {
    if VP_INDEX < 0 {
        panic!("Viewport_GetSize: Viewport stack is empty");
    }
    out.x = VP[VP_INDEX as usize].sx;
    out.y = VP[VP_INDEX as usize].sy;
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Push(x: i32, y: i32, sx: i32, sy: i32, isWindow: bool) {
    if VP_INDEX + 1 >= 16 {
        panic!("Viewport_Push: Maximum viewport stack depth exceeded");
    }

    VP_INDEX += 1;

    let this: &mut VP = &mut VP[VP_INDEX as usize];
    this.x = x;
    this.y = y;
    this.sx = sx;
    this.sy = sy;
    this.isWindow = isWindow;

    // Set up the ortho projection matrix for UI elements.
    let ortho_proj = if this.isWindow {
        Matrix::from_translation(vec3(-1.0, 1.0, 0.0))
            * Matrix::from_scale(vec3(2.0f32 / this.sx as f32, -2.0f32 / this.sy as f32, 1.0))
    } else {
        Matrix::from_translation(vec3(-1.0, -1.0, 0.0))
            * Matrix::from_scale(vec3(2.0f32 / this.sx as f32, 2.0f32 / this.sy as f32, 1.0))
    };
    ShaderVar::push_matrix("mProjUI", &ortho_proj);
    ShaderVar::push_matrix("mWorldViewUI", &Matrix::IDENTITY);

    glcheck!(gl::Viewport(this.x, this.y, this.sx, this.sy));
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Pop() {
    if VP_INDEX < 0 {
        panic!("Viewport_Pop: Viewport stack is empty");
    }

    ShaderVar::pop("mWorldViewUI");
    ShaderVar::pop("mProjUI");

    VP_INDEX -= 1;
    if VP_INDEX >= 0 {
        let viewport = &VP[VP_INDEX as usize];
        glcheck!(gl::Viewport(
            viewport.x,
            viewport.y,
            viewport.sx,
            viewport.sy
        ));
    }
}
