use crate::internal::Memory::*;
use crate::RenderState::*;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);
    fn glCullFace(mode: GLenum);
    fn glDepthFunc(func: GLenum);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glGetError() -> GLenum;
    fn glHint(target: GLenum, mode: GLenum);
    fn glLineWidth(width: GLfloat);
    fn glLoadIdentity();
    fn glMatrixMode(mode: GLenum);
    fn glPixelStorei(pname: GLenum, param: GLint);
    fn glewInit() -> GLenum;
}

pub type GLenum = u32;
pub type GLint = i32;
pub type GLfloat = f32;
#[no_mangle]
pub unsafe extern "C" fn OpenGL_Init() {
    static mut init: bool = 0 as i32 != 0;
    if !init {
        init = 1 as i32 != 0;
        glewInit();
    }
    glDisable(0x809d as i32 as GLenum);
    glDisable(0xb44 as i32 as GLenum);
    glCullFace(0x405 as i32 as GLenum);
    glPixelStorei(0xd05 as i32 as GLenum, 1 as i32);
    glPixelStorei(0xcf5 as i32 as GLenum, 1 as i32);
    glDepthFunc(0x203 as i32 as GLenum);
    glEnable(0xbe2 as i32 as GLenum);
    glBlendFunc(1 as i32 as GLenum, 0 as i32 as GLenum);
    glEnable(0x884f as i32 as GLenum);
    glDisable(0xb10 as i32 as GLenum);
    glDisable(0xb20 as i32 as GLenum);
    glHint(0xc51 as i32 as GLenum, 0x1101 as i32 as GLenum);
    glHint(0xc52 as i32 as GLenum, 0x1101 as i32 as GLenum);
    glLineWidth(2 as i32 as GLfloat);
    glMatrixMode(0x1701 as i32 as GLenum);
    glLoadIdentity();
    glMatrixMode(0x1700 as i32 as GLenum);
    glLoadIdentity();
    RenderState_PushAllDefaults();
}
#[no_mangle]
pub unsafe extern "C" fn OpenGL_CheckError(mut file: *const libc::c_char, mut line: i32) {
    let mut errorID: GLenum = glGetError();
    let mut error: *const libc::c_char = 0 as *const libc::c_char;
    match errorID {
        0 => return,
        1280 => {
            error = b"GL_INVALID_ENUM\0" as *const u8 as *const libc::c_char;
        }
        1281 => {
            error = b"GL_INVALID_VALUE\0" as *const u8 as *const libc::c_char;
        }
        1282 => {
            error = b"GL_INVALID_OPERATION\0" as *const u8 as *const libc::c_char;
        }
        1286 => {
            error = b"GL_INVALID_FRAMEBUFFER_OPERATION\0" as *const u8 as *const libc::c_char;
        }
        1285 => {
            error = b"GL_OUT_OF_MEMORY\0" as *const u8 as *const libc::c_char;
        }
        _ => {
            Fatal(
                b"OpenGL_CheckError: glGetError returned illegal error code %u at %s:%d\0"
                    as *const u8 as *const libc::c_char,
                errorID,
                file,
                line,
            );
        }
    }
    Fatal(
        b"OpenGL_CheckError: %s at %s:%d\0" as *const u8 as *const libc::c_char,
        error,
        file,
        line,
    );
}
