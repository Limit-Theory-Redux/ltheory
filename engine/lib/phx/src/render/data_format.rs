#[luajit_ffi_gen::luajit_ffi(with_impl = true)]
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    I8 = 0x1400,    // gl::BYTE
    U8 = 0x1401,    // gl::UNSIGNED_BYTE
    I16 = 0x1402,   // gl::SHORT
    U16 = 0x1403,   // gl::UNSIGNED_SHORT
    I32 = 0x1404,   // gl::INT
    U32 = 0x1405,   // gl::UNSIGNED_INT
    Float = 0x1406, // gl::FLOAT
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
