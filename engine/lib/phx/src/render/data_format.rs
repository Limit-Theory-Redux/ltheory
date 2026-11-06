#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    Float,
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
