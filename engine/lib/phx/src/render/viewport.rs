use super::{gl, *};
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
static mut VP_INDEX: i32 = -1;

static mut VP: [VP; 16] = [VP {
    x: 0,
    y: 0,
    sx: 0,
    sy: 0,
    isWindow: false,
}; 16];

extern "C" fn Viewport_Set(this: &VP) {
    glcheck!(gl::Viewport(this.x, this.y, this.sx, this.sy));
    glcheck!(gl::MatrixMode(gl::PROJECTION));
    glcheck!(gl::LoadIdentity());

    if this.isWindow {
        glcheck!(gl::Translatef(-1.0f32, 1.0f32, 0.0f32));
        glcheck!(gl::Scalef(
            2.0f32 / this.sx as f32,
            -2.0f32 / this.sy as f32,
            1.0f32
        ));
    } else {
        glcheck!(gl::Translatef(-1.0f32, -1.0f32, 0.0f32));
        glcheck!(gl::Scalef(
            2.0f32 / this.sx as f32,
            2.0f32 / this.sy as f32,
            1.0f32
        ));
    };
}

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
    let this: *mut VP = VP.as_mut_ptr().offset(VP_INDEX as isize);
    (*this).x = x;
    (*this).y = y;
    (*this).sx = sx;
    (*this).sy = sy;
    (*this).isWindow = isWindow;
    Viewport_Set(&*this);
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Pop() {
    if VP_INDEX < 0 {
        panic!("Viewport_Pop: Viewport stack is empty");
    }
    VP_INDEX -= 1;
    if VP_INDEX >= 0 {
        Viewport_Set(&*VP.as_mut_ptr().offset(VP_INDEX as isize));
    }
}
