pub mod gl {
    #![allow(clippy::missing_transmute_annotations)] // disable clippy warnings in the generated bindings.rs file
    #![allow(unsafe_code)] // TODO: remove

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[allow(clippy::unused_unit)]
    pub fn error_to_str(err: types::GLenum) -> &'static str {
        match err {
            INVALID_ENUM => "GL_INVALID_ENUM",
            INVALID_VALUE => "GL_INVALID_VALUE",
            INVALID_OPERATION => "GL_INVALID_OPERATION",
            INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            _ => "unknown error",
        }
    }
}

macro_rules! glcheck {
    ($s:stmt) => {{
        #[allow(unsafe_code)] // TODO: remove
        let result = unsafe { $s };
        // Uncomment this to enable GL checks.
        /*
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
        */
        result
    }};
}

pub(crate) use glcheck;
