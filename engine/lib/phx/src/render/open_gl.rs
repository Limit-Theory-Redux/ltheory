use super::*;
use crate::common::*;

use std::ffi::{CStr, CString};

use glutin::{display::GetGlDisplay, prelude::GlDisplay};

#[no_mangle]
pub unsafe extern "C" fn OpenGL_Init(gl_config: glutin::config::Config) {
    static mut init: bool = false;
    if !init {
        init = true;
        gl::load_with(|s| {
            let cs = CString::new(s.as_bytes()).unwrap();
            gl_config.display().get_proc_address(cs.as_c_str())
        });
    }

    gl::Disable(gl::MULTISAMPLE);
    gl::Disable(gl::CULL_FACE);
    gl::CullFace(gl::BACK);

    gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::DepthFunc(gl::LEQUAL);

    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::ONE, gl::ZERO);

    gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
    gl::Disable(gl::POINT_SMOOTH);
    gl::Disable(gl::LINE_SMOOTH);
    gl::Hint(gl::POINT_SMOOTH_HINT, gl::FASTEST);
    gl::Hint(gl::LINE_SMOOTH_HINT, gl::FASTEST);
    gl::LineWidth(2.0f32);

    gl::MatrixMode(gl::PROJECTION);
    gl::LoadIdentity();
    gl::MatrixMode(gl::MODELVIEW);
    gl::LoadIdentity();

    RenderState_PushAllDefaults();
}

#[no_mangle]
pub unsafe extern "C" fn OpenGL_CheckError(file: *const libc::c_char, line: i32) {
    let errorID: gl::types::GLenum = gl::GetError();
    let error = match errorID {
        0 => return,
        1280 => "GL_INVALID_ENUM",
        1281 => "GL_INVALID_VALUE",
        1282 => "GL_INVALID_OPERATION",
        1286 => "GL_INVALID_FRAMEBUFFER_OPERATION",
        1285 => "GL_OUT_OF_MEMORY",
        _ => {
            panic!(
                "OpenGL_CheckError: gl::GetError returned illegal error code {errorID} at {:?}:{line}",
                CStr::from_ptr(file),
            );
        }
    };

    panic!(
        "OpenGL_CheckError: {error} at {:?}:{line}",
        CStr::from_ptr(file)
    );
}
