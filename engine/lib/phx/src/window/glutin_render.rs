use std::ffi::{CStr, CString};

use glutin::prelude::GlDisplay;
use tracing::debug;

use crate::render::{gl, RenderState_PushAllDefaults};

pub fn init_renderer<D: GlDisplay>(gl_display: &D) {
    debug!("Init renderer");

    unsafe {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if let Some(renderer) = get_gl_string(gl::RENDERER) {
            println!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(gl::VERSION) {
            println!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
            println!("Shaders version on {}", shaders_version.to_string_lossy());
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

pub fn resize(width: i32, height: i32) {
    unsafe {
        gl::Viewport(0, 0, width, height);
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

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}
