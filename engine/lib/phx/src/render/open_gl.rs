use super::*;
use crate::common::*;

use std::ffi::{CStr, CString};

use glutin::{display::GetGlDisplay, prelude::GlDisplay};
use tracing::debug;

pub struct OpenGL;

impl OpenGL {
    pub fn init(gl_config: glutin::config::Config) {
        debug!("OpenGL::init");

        unsafe {
            static mut init: bool = false;
            if !init {
                init = true;

                gl::load_with(|s| {
                    let cs = CString::new(s.as_bytes()).unwrap();
                    let addr = gl_config.display().get_proc_address(cs.as_c_str());

                    // debug!("Load OpenGL function: {s}. Address: {addr:?}");

                    addr
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
    }

    pub fn check_error(file: &str, line: u32) {
        let errorID: gl::types::GLenum = unsafe { gl::GetError() };
        let error = match errorID {
            0 => return,
            1280 => "GL_INVALID_ENUM",
            1281 => "GL_INVALID_VALUE",
            1282 => "GL_INVALID_OPERATION",
            1286 => "GL_INVALID_FRAMEBUFFER_OPERATION",
            1285 => "GL_OUT_OF_MEMORY",
            _ => {
                panic!(
                    "OpenGL_CheckError: gl::GetError returned illegal error code {errorID} at {file}:{line}"
                );
            }
        };

        panic!("OpenGL_CheckError: {error} at {file}:{line}");
    }
}
