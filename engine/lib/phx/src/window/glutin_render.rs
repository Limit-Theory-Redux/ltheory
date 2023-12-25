use std::ffi::CString;

use glutin::prelude::GlDisplay;
use tracing::{debug, info, warn};

use crate::render::*;

pub fn init_renderer<D: GlDisplay>(gl_display: &D) {
    debug!("Init renderer");

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    if let Some(vendor) = gl_get_string(gl::VENDOR) {
        info!("OpenGL Vendor: {vendor}");
    } else {
        warn!("No OpenGL vendor info");
    }

    if let Some(renderer) = gl_get_string(gl::RENDERER) {
        info!("Running on {renderer}");
    } else {
        warn!("No renderer info");
    }

    if let Some(version) = gl_get_string(gl::VERSION) {
        info!("OpenGL Version {version}");
    } else {
        warn!("No OpenGL Version info");
    }

    if let Some(shaders_version) = gl_get_string(gl::SHADING_LANGUAGE_VERSION) {
        info!("Shaders version on {shaders_version}");
    } else {
        warn!("No Shaders version info");
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
    gl_line_width(2.0f32);

    gl_matrix_mode(gl::PROJECTION);
    gl_load_identity();
    gl_matrix_mode(gl::MODELVIEW);
    gl_load_identity();

    unsafe { RenderState_PushAllDefaults() };
}

pub fn resize(width: i32, height: i32) {
    gl_viewport(0, 0, width, height);
}
