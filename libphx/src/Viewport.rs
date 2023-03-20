use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use crate::GL::gl;
use libc;

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

unsafe extern "C" fn Viewport_Set(mut this: *const VP) {
    gl::Viewport((*this).x, (*this).y, (*this).sx, (*this).sy);
    gl::MatrixMode(gl::PROJECTION);
    gl::LoadIdentity();
    if (*this).isWindow {
        gl::Translatef(-1.0f32, 1.0f32, 0.0f32);
        gl::Scalef(
            2.0f32 / (*this).sx as f32,
            -2.0f32 / (*this).sy as f32,
            1.0f32,
        );
    } else {
        gl::Translatef(-1.0f32, -1.0f32, 0.0f32);
        gl::Scalef(
            2.0f32 / (*this).sx as f32,
            2.0f32 / (*this).sy as f32,
            1.0f32,
        );
    };
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetAspect() -> f32 {
    if vpIndex < 0 {
        Fatal(b"Viewport_GetAspect: Viewport stack is empty\0" as *const u8 as *const libc::c_char);
    }
    vp[vpIndex as usize].sx as f32 / vp[vpIndex as usize].sy as f32
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(mut out: *mut IVec2) {
    if vpIndex < 0 {
        Fatal(b"Viewport_GetSize: Viewport stack is empty\0" as *const u8 as *const libc::c_char);
    }
    (*out).x = vp[vpIndex as usize].sx;
    (*out).y = vp[vpIndex as usize].sy;
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Push(
    mut x: i32,
    mut y: i32,
    mut sx: i32,
    mut sy: i32,
    mut isWindow: bool,
) {
    if vpIndex + 1 >= 16 {
        Fatal(
            b"Viewport_Push: Maximum viewport stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    vpIndex += 1;
    let mut this: *mut VP = vp.as_mut_ptr().offset(vpIndex as isize);
    (*this).x = x;
    (*this).y = y;
    (*this).sx = sx;
    (*this).sy = sy;
    (*this).isWindow = isWindow;
    Viewport_Set(this);
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_Pop() {
    if vpIndex < 0 {
        Fatal(b"Viewport_Pop: Viewport stack is empty\0" as *const u8 as *const libc::c_char);
    }
    vpIndex -= 1;
    if vpIndex >= 0 {
        Viewport_Set(vp.as_mut_ptr().offset(vpIndex as isize));
    }
}
