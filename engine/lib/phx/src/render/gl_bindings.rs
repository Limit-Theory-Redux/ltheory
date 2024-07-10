pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub fn error_to_str(err: types::GLenum) -> &'static str {
        match err {
            INVALID_ENUM => "GL_INVALID_ENUM",
            INVALID_VALUE => "GL_INVALID_VALUE",
            INVALID_OPERATION => "GL_INVALID_OPERATION",
            INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            _ => "unknown error",
        }
    }
}

macro_rules! glcheck {
    ($s:stmt) => {{
        let result = unsafe { $s };
        if cfg!(debug_assertions) {
            let err = unsafe { gl::GetError() };
            if err != gl::NO_ERROR {
                crate::logging::error!(
                    "{}:{} - OpenGL check error, {} caused {}",
                    file!(),
                    line!(),
                    stringify!($s),
                    gl::error_to_str(err),
                );
            }
        }
        result
    }};
}

pub(crate) use glcheck;
