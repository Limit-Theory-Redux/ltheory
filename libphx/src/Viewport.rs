use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
extern "C" {
    fn glLoadIdentity();
    fn glMatrixMode(mode: GLenum);
    fn glScalef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glTranslatef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;

pub type GLenum = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type GLfloat = f32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VP {
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub sx: libc::c_int,
    pub sy: libc::c_int,
    pub isWindow: bool,
}
static mut vpIndex: libc::c_int = -(1 as libc::c_int);
static mut vp: [VP; 16] = [VP {
    x: 0,
    y: 0,
    sx: 0,
    sy: 0,
    isWindow: false,
}; 16];
unsafe extern "C" fn Viewport_Set(mut this: *const VP) {
    glViewport((*this).x, (*this).y, (*this).sx, (*this).sy);
    glMatrixMode(0x1701 as libc::c_int as GLenum);
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
    if vpIndex < 0 as libc::c_int {
        Fatal(
            b"Viewport_GetAspect: Viewport stack is empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    return vp[vpIndex as usize].sx as f32
        / vp[vpIndex as usize].sy as f32;
}
#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(mut out: *mut IVec2) {
    if vpIndex < 0 as libc::c_int {
        Fatal(
            b"Viewport_GetSize: Viewport stack is empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    (*out).x = vp[vpIndex as usize].sx;
    (*out).y = vp[vpIndex as usize].sy;
}
#[no_mangle]
pub unsafe extern "C" fn Viewport_Push(
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut isWindow: bool,
) {
    if vpIndex + 1 as libc::c_int >= 16 as libc::c_int {
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
    if vpIndex < 0 as libc::c_int {
        Fatal(
            b"Viewport_Pop: Viewport stack is empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    vpIndex -= 1;
    if vpIndex >= 0 as libc::c_int {
        Viewport_Set(vp.as_mut_ptr().offset(vpIndex as isize));
    }
}
