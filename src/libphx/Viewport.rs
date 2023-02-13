use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn glLoadIdentity();
    fn glMatrixMode(mode: GLenum);
    fn glScalef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glTranslatef(x: GLfloat, y: GLfloat, z: GLfloat);
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
}
pub type GLenum = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type GLfloat = libc::c_float;
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
unsafe extern "C" fn Viewport_Set(mut self_0: *const VP) {
    glViewport((*self_0).x, (*self_0).y, (*self_0).sx, (*self_0).sy);
    glMatrixMode(0x1701 as libc::c_int as GLenum);
    glLoadIdentity();
    if (*self_0).isWindow {
        glTranslatef(-1.0f64 as GLfloat, 1.0f64 as GLfloat, 0.0f64 as GLfloat);
        glScalef(
            2.0f32 / (*self_0).sx as libc::c_float,
            -2.0f32 / (*self_0).sy as libc::c_float,
            1.0f32,
        );
    } else {
        glTranslatef(-1.0f64 as GLfloat, -1.0f64 as GLfloat, 0.0f64 as GLfloat);
        glScalef(
            2.0f32 / (*self_0).sx as libc::c_float,
            2.0f32 / (*self_0).sy as libc::c_float,
            1.0f32,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Viewport_GetAspect() -> libc::c_float {
    if vpIndex < 0 as libc::c_int {
        Fatal(
            b"Viewport_GetAspect: Viewport stack is empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    return vp[vpIndex as usize].sx as libc::c_float
        / vp[vpIndex as usize].sy as libc::c_float;
}
#[no_mangle]
pub unsafe extern "C" fn Viewport_GetSize(mut out: *mut Vec2i) {
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
    let mut self_0: *mut VP = vp.as_mut_ptr().offset(vpIndex as isize);
    (*self_0).x = x;
    (*self_0).y = y;
    (*self_0).sx = sx;
    (*self_0).sy = sy;
    (*self_0).isWindow = isWindow;
    Viewport_Set(self_0);
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
