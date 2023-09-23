use std::ffi::{CStr, CString};

use glutin::prelude::GlDisplay;
use tracing::debug;

use crate::render::*;

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

        gl_disable(gl::MULTISAMPLE);
        gl_disable(gl::CULL_FACE);
        gl_cull_face(gl::BACK);

        gl_pixel_storei(gl::PACK_ALIGNMENT, 1);
        gl_pixel_storei(gl::UNPACK_ALIGNMENT, 1);
        gl_depth_func(gl::LEQUAL);

        gl_enable(gl::BLEND);
        gl_blend_func(gl::ONE, gl::ZERO);

        gl_enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
        gl_disable(gl::POINT_SMOOTH);
        gl_disable(gl::LINE_SMOOTH);
        gl_hint(gl::POINT_SMOOTH_HINT, gl::FASTEST);
        gl_hint(gl::LINE_SMOOTH_HINT, gl::FASTEST);
        gl::LineWidth(2.0f32);

        gl_matrix_mode(gl::PROJECTION);
        gl::LoadIdentity();
        gl_matrix_mode(gl::MODELVIEW);
        gl::LoadIdentity();

        RenderState_PushAllDefaults();
    }
}

pub fn resize(width: i32, height: i32) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}
