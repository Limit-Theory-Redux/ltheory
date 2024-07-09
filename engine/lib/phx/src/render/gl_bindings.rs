pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

macro_rules! glcheck {
    ($s:stmt) => {{
        let result = unsafe { $s };
        if cfg!(debug_assertions) {
            let err = unsafe { gl::GetError() };
            if err != gl::NO_ERROR {
                let err_str = match err {
                    gl::INVALID_ENUM => "GL_INVALID_ENUM",
                    gl::INVALID_VALUE => "GL_INVALID_VALUE",
                    gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                    gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                    gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                    gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                    gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                    _ => "unknown error",
                };
                crate::logging::error!(
                    "{}:{} - OpenGL check error, {} caused {}",
                    file!(),
                    line!(),
                    stringify!($s),
                    err_str
                );
            }
        }
        result
    }};
}

pub(crate) use glcheck;
