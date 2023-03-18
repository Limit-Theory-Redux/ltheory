use crate::internal::Memory::*;
use glam::IVec2;
use glam::Vec3;
use libc;

extern "C" {
    fn glLoadIdentity();
    fn glMatrixMode(mode: GLenum);
    fn glScalef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glTranslatef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    fn Fatal(_: *const libc::c_char, _: ...);
}

pub type GLenum = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLfloat = f32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VP {
    pub x: i32,
    pub y: i32,
    pub sx: i32,
    pub sy: i32,
    pub isWindow: bool,
}
static mut vpIndex: i32 = -1_i32;

static mut vp: [VP; 16] = [VP {
    x: 0,
    y: 0,
    sx: 0,
    sy: 0,
    isWindow: false,
}; 16];
unsafe extern "C" fn Viewport_Set(mut this: *const VP) {
    glViewport((*this).x, (*this).y, (*this).sx, (*this).sy);
    glMatrixMode(0x1701_i32 as GLenum);
    glLoadIdentity();
    if (*this).isWindow {
        glTranslatef(-1.0f64 as GLfloat, 1.0f64 as GLfloat, 0.0f64 as GLfloat);
        glScalef(
            2.0f32 / (*this).sx as f32,
            -2.0f32 / (*this).sy as f32,
            1.0f32,
        );
    } else {
        glTranslatef(-1.0f64 as GLfloat, -1.0f64 as GLfloat, 0.0f64 as GLfloat);
        glScalef(
            2.0f32 / (*this).sx as f32,
            2.0f32 / (*this).sy as f32,
            1.0f32,
        );
    };
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetAspect() -> f32 {
    if vpIndex < 0_i32 {
        Fatal(b"Viewport_GetAspect: Viewport stack is empty\0" as *const u8 as *const libc::c_char);
    }
    vp[vpIndex as usize].sx as f32 / vp[vpIndex as usize].sy as f32
}

#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(mut out: *mut IVec2) {
    if vpIndex < 0_i32 {
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
    if vpIndex + 1_i32 >= 16_i32 {
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
    if vpIndex < 0_i32 {
        Fatal(b"Viewport_Pop: Viewport stack is empty\0" as *const u8 as *const libc::c_char);
    }
    vpIndex -= 1;
    if vpIndex >= 0_i32 {
        Viewport_Set(vp.as_mut_ptr().offset(vpIndex as isize));
    }
}
