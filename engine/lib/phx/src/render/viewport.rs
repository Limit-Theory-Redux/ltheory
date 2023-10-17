use super::{gl, *};
use crate::common::*;
use crate::math::IVec2;

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
static mut vpIndex: i32 = -1;

static mut vp: [VP; 16] = [VP {
    x: 0,
    y: 0,
    sx: 0,
    sy: 0,
    isWindow: false,
}; 16];

extern "C" fn Viewport_Set(this: &VP) {
    gl_viewport(this.x, this.y, this.sx, this.sy);
    gl_matrix_mode(gl::PROJECTION);
    gl_load_identity();

    if this.isWindow {
        gl_translatef(-1.0f32, 1.0f32, 0.0f32);
        gl_scalef(2.0f32 / this.sx as f32, -2.0f32 / this.sy as f32, 1.0f32);
    } else {
        gl_translatef(-1.0f32, -1.0f32, 0.0f32);
        gl_scalef(2.0f32 / this.sx as f32, 2.0f32 / this.sy as f32, 1.0f32);
    };
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetAspect() -> f32 {
    if vpIndex < 0 {
        panic!("Viewport_GetAspect: Viewport stack is empty");
    }
    vp[vpIndex as usize].sx as f32 / vp[vpIndex as usize].sy as f32
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(out: &mut IVec2) {
    if vpIndex < 0 {
        panic!("Viewport_GetSize: Viewport stack is empty");
    }
    out.x = vp[vpIndex as usize].sx;
    out.y = vp[vpIndex as usize].sy;
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Push(x: i32, y: i32, sx: i32, sy: i32, isWindow: bool) {
    if vpIndex + 1 >= 16 {
        panic!("Viewport_Push: Maximum viewport stack depth exceeded");
    }
    vpIndex += 1;
    let this: *mut VP = vp.as_mut_ptr().offset(vpIndex as isize);
    (*this).x = x;
    (*this).y = y;
    (*this).sx = sx;
    (*this).sy = sy;
    (*this).isWindow = isWindow;
    Viewport_Set(&mut *this);
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Pop() {
    if vpIndex < 0 {
        panic!("Viewport_Pop: Viewport stack is empty");
    }
    vpIndex -= 1;
    if vpIndex >= 0 {
        Viewport_Set(&mut *vp.as_mut_ptr().offset(vpIndex as isize));
    }
}
