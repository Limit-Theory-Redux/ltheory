use ::libc;
extern "C" {
    fn Fatal(_: cstr, _: ...);
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
    fn RenderState_PushAllDefaults();
}
pub type cstr = *const libc::c_char;
pub type GLenum = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLfloat = libc::c_float;
#[no_mangle]
pub unsafe extern "C" fn OpenGL_Init() {
    static mut init: bool = 0 as libc::c_int != 0;
    if !init {
        init = 1 as libc::c_int != 0;
        glewInit();
    }
    glDisable(0x809d as libc::c_int as GLenum);
    glDisable(0xb44 as libc::c_int as GLenum);
    glCullFace(0x405 as libc::c_int as GLenum);
    glPixelStorei(0xd05 as libc::c_int as GLenum, 1 as libc::c_int);
    glPixelStorei(0xcf5 as libc::c_int as GLenum, 1 as libc::c_int);
    glDepthFunc(0x203 as libc::c_int as GLenum);
    glEnable(0xbe2 as libc::c_int as GLenum);
    glBlendFunc(1 as libc::c_int as GLenum, 0 as libc::c_int as GLenum);
    glEnable(0x884f as libc::c_int as GLenum);
    glDisable(0xb10 as libc::c_int as GLenum);
    glDisable(0xb20 as libc::c_int as GLenum);
    glHint(0xc51 as libc::c_int as GLenum, 0x1101 as libc::c_int as GLenum);
    glHint(0xc52 as libc::c_int as GLenum, 0x1101 as libc::c_int as GLenum);
    glLineWidth(2 as libc::c_int as GLfloat);
    glMatrixMode(0x1701 as libc::c_int as GLenum);
    glLoadIdentity();
    glMatrixMode(0x1700 as libc::c_int as GLenum);
    glLoadIdentity();
    RenderState_PushAllDefaults();
}
#[no_mangle]
pub unsafe extern "C" fn OpenGL_CheckError(mut file: cstr, mut line: libc::c_int) {
    let mut errorID: GLenum = glGetError();
    let mut error: cstr = 0 as cstr;
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
            error = b"GL_INVALID_FRAMEBUFFER_OPERATION\0" as *const u8
                as *const libc::c_char;
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
