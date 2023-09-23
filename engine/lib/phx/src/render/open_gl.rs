use super::*;
use crate::common::*;
use gl::types::*;

use std::ffi::{CStr, CString};

use glutin::{display::GetGlDisplay, prelude::GlDisplay};
use tracing::{debug, error};

pub fn check_error(file: &str, line: u32, msg: &str) {
    let msg_str = if !msg.is_empty() {
        format!(", Message: {msg}")
    } else {
        String::new()
    };

    let errorID = unsafe { gl::GetError() };
    let error = match errorID {
        gl::NO_ERROR => return,
        gl::INVALID_ENUM => "GL_INVALID_ENUM".into(),
        gl::INVALID_VALUE => "GL_INVALID_VALUE".into(),
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION".into(),
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY".into(),
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW".into(),
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW".into(),
        _ => format!("gl::GetError returned unknown error code {errorID}"),
    };

    error!("OpenGL_CheckError: {error} at {file}:{line}{msg_str}");
}

macro_rules! gl_error_check {
    ($name:ident, $msg:expr) => {
        // NOTE: uncomment next 3 lines to catch OpenGl errors (for debugging purposes only - heavily impacts performance)
        // let caller_location = std::panic::Location::caller();
        // let msg_str = format!("{}({})", stringify!($name), $msg);
        // check_error(caller_location.file(), caller_location.line(), &msg_str);
    };
}

macro_rules! gl_func {
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*) -> $ret:ty, $msg:expr) => {
        #[inline]
        #[track_caller]
        pub fn $name($($param_name: $param_ty),*) -> $ret {
            let res = unsafe { gl::$gl_name($($param_name),*) };

            gl_error_check!($name, $msg);

            res
        }
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*) -> $ret:ty) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> $ret, "");
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*), $msg:expr) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> (), $msg);
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*)) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> (), "");
    };
}

#[inline]
#[track_caller]
pub fn gl_get_string(name: GLenum) -> Option<String> {
    unsafe {
        let s = gl::GetString(name);

        gl_error_check!(gl_get_string, "");

        (!s.is_null()).then(|| CStr::from_ptr(s.cast()).to_string_lossy().to_string())
    }
}

gl_func!(Enable, gl_enable(cap: GLenum));
gl_func!(Disable, gl_disable(cap: GLenum), &format!("cap = {cap}"));
gl_func!(Begin, gl_begin(mode: GLenum));
gl_func!(End, gl_end());
gl_func!(CheckFramebufferStatus, gl_check_framebuffer_status(target: GLenum) -> GLenum);
gl_func!(Clear, gl_clear(mask: GLbitfield));
gl_func!(Hint, gl_hint(target: GLenum, mode: GLenum));
gl_func!(MatrixMode, gl_matrix_mode(mode: GLenum));
gl_func!(BindBuffer, gl_bind_buffer(target: GLenum, buffer: GLuint));
gl_func!(
    GetIntegerv,
    gl_get_integerv(pname: GLenum, data: &mut GLint)
);
gl_func!(GetFloatv, gl_get_floatv(pname: GLenum, data: *mut GLfloat));
gl_func!(
    BlendFuncSeparate,
    gl_blend_func_separate(
        sfactor_rgb: GLenum,
        dfactor_rgb: GLenum,
        sfactor_alpha: GLenum,
        dfactor_alpha: GLenum
    )
);
gl_func!(BlendFunc, gl_blend_func(sfactor: GLenum, dfactor: GLenum));
gl_func!(CullFace, gl_cull_face(mode: GLenum));
gl_func!(DepthMask, gl_depth_mask(flag: GLboolean));
gl_func!(PolygonMode, gl_polygon_mode(face: GLenum, mode: GLenum));
gl_func!(
    BindFramebuffer,
    gl_bind_framebuffer(target: GLenum, framebuffer: GLuint)
);
gl_func!(
    FramebufferTexture2D,
    gl_framebuffer_texture2d(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint
    )
);
gl_func!(
    FramebufferTexture3D,
    gl_framebuffer_texture3d(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
        zoffset: GLint
    )
);
gl_func!(
    GetShaderiv,
    gl_get_shaderiv(shader: GLuint, pname: GLenum, params: &mut GLint)
);
gl_func!(
    GetProgramiv,
    gl_get_programiv(program: GLuint, pname: GLenum, params: &mut GLint)
);
gl_func!(ActiveTexture, gl_active_texture(texture: GLenum));
gl_func!(
    BindTexture,
    gl_bind_texture(target: GLenum, texture: GLuint)
);
gl_func!(
    TexParameteri,
    gl_tex_parameteri(target: GLenum, pname: GLenum, param: GLint)
);
gl_func!(
    TexParameterf,
    gl_tex_parameterf(target: GLenum, pname: GLenum, param: GLfloat)
);
gl_func!(GenerateMipmap, gl_generate_mipmap(target: GLenum));
gl_func!(PixelStorei, gl_pixel_storei(pname: GLenum, param: GLint));
gl_func!(DepthFunc, gl_depth_func(func: GLenum));
gl_func!(
    GetShaderInfoLog,
    gl_get_shader_info_log(
        shader: GLuint,
        buf_size: GLsizei,
        length: *mut GLsizei,
        info_log: *mut GLchar
    )
);
gl_func!(
    GetProgramInfoLog,
    gl_get_program_info_log(
        program: GLuint,
        buf_size: GLsizei,
        length: *mut GLsizei,
        info_log: *mut GLchar
    )
);
