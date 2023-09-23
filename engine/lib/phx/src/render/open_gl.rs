use super::*;
use crate::common::*;
use gl::types::*;

use std::ffi::{CStr, CString};

use glutin::{display::GetGlDisplay, prelude::GlDisplay};
use tracing::debug;

pub fn check_error(file: &str, line: u32, msg: &str) {
    let msg_str = if !msg.is_empty() {
        format!("\nMessage: {msg}")
    } else {
        String::new()
    };

    let errorID = unsafe { gl::GetError() };
    let error = match errorID {
        0 => return,
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        _ => {
            panic!(
                "OpenGL_CheckError: gl::GetError returned unknown error code {errorID} at {file}:{line}{msg_str}"
            );
        }
    };

    panic!("OpenGL_CheckError: {error} at {file}:{line}{msg_str}");
}

macro_rules! gl_error_check {
    ($msg:expr) => {
        // NOTE: uncomment next 2 lines to catch OpenGl errors
        // let caller_location = std::panic::Location::caller();
        // check_error(caller_location.file(), caller_location.line(), $msg);
    };
}

macro_rules! gl_func {
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*) -> $ret:ty, $msg:expr) => {
        #[inline]
        #[track_caller]
        pub fn $name($($param_name: $param_ty),*) -> $ret {
            let res = unsafe { gl::$gl_name($($param_name),*) };

            gl_error_check!($msg);

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

gl_func!(Enable, gl_enable(cap: GLenum));
gl_func!(Disable, gl_disable(cap: GLenum));
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
