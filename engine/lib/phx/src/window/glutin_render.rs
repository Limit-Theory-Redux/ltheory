use std::ffi::{CStr, CString};

use glutin::prelude::GlDisplay;
use tracing::{debug, info, warn};

use crate::render::*;

pub fn init_renderer<D: GlDisplay>(gl_display: &D) {
    debug!("Init renderer");

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    let gl_get_string = |name: gl::types::GLenum| -> Option<String> {
        unsafe {
            let s = gl::GetString(name);
            (!s.is_null()).then(|| CStr::from_ptr(s.cast()).to_string_lossy().to_string())
        }
    };

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

    glcheck!(gl::Disable(gl::MULTISAMPLE));
    glcheck!(gl::Disable(gl::CULL_FACE));
    glcheck!(gl::CullFace(gl::BACK));

    glcheck!(gl::PixelStorei(gl::PACK_ALIGNMENT, 1));
    glcheck!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1));
    glcheck!(gl::DepthFunc(gl::LEQUAL));

    glcheck!(gl::Enable(gl::BLEND));
    glcheck!(gl::BlendFunc(gl::ONE, gl::ZERO));

    glcheck!(gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS));
    glcheck!(gl::Disable(gl::LINE_SMOOTH));
    glcheck!(gl::Hint(gl::LINE_SMOOTH_HINT, gl::FASTEST));
    glcheck!(gl::LineWidth(2.0f32));

    glcheck!(gl::MatrixMode(gl::PROJECTION));
    glcheck!(gl::LoadIdentity());
    glcheck!(gl::MatrixMode(gl::MODELVIEW));
    glcheck!(gl::LoadIdentity());

    unsafe { RenderState_PushAllDefaults() };
}

pub fn resize(width: i32, height: i32) {
    glcheck!(gl::Viewport(0, 0, width, height));
}
