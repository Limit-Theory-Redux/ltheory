use super::gl;

#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    I8 = gl::BYTE,
    U8 = gl::UNSIGNED_BYTE,
    I16 = gl::SHORT,
    U16 = gl::UNSIGNED_SHORT,
    I32 = gl::INT,
    U32 = gl::UNSIGNED_INT,
    Float = gl::FLOAT,
}

#[luajit_ffi_gen::luajit_ffi]
impl DataFormat {
    /// Size in bytes of single element
    pub fn get_size(this: DataFormat) -> i32 {
        match this {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::Float => 4,
        }
    }
}
