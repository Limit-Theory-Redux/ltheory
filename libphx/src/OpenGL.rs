use crate::internal::Memory::*;
use crate::Math::Vec3;
use crate::RenderState::*;
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
    static mut init: bool = false;
    if !init {
        init = true;
        glewInit();
    }
    glDisable(0x809d_i32 as GLenum);
    glDisable(0xb44_i32 as GLenum);
    glCullFace(0x405_i32 as GLenum);
    glPixelStorei(0xd05_i32 as GLenum, 1_i32);
    glPixelStorei(0xcf5_i32 as GLenum, 1_i32);
    glDepthFunc(0x203_i32 as GLenum);
    glEnable(0xbe2_i32 as GLenum);
    glBlendFunc(1_i32 as GLenum, 0_i32 as GLenum);
    glEnable(0x884f_i32 as GLenum);
    glDisable(0xb10_i32 as GLenum);
    glDisable(0xb20_i32 as GLenum);
    glHint(0xc51_i32 as GLenum, 0x1101_i32 as GLenum);
    glHint(0xc52_i32 as GLenum, 0x1101_i32 as GLenum);
    glLineWidth(2_i32 as GLfloat);
    glMatrixMode(0x1701_i32 as GLenum);
    glLoadIdentity();
    glMatrixMode(0x1700_i32 as GLenum);
    glLoadIdentity();
    RenderState_PushAllDefaults();
}

#[no_mangle]
pub unsafe extern "C" fn OpenGL_CheckError(mut file: *const libc::c_char, mut line: i32) {
    let mut errorID: GLenum = glGetError();
    let mut error: *const libc::c_char = std::ptr::null();
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
