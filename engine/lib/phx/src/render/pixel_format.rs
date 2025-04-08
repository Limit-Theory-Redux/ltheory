#[luajit_ffi_gen::luajit_ffi(with_impl = true)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    DepthComponent = 0x1902, // gl::DEPTH_COMPONENT
    Red = 0x1903,            // gl::RED
    RGB = 0x1907,            // gl::RGB
    RGBA = 0x1908,           // gl::RGBA
    BGR = 0x80E0,            // gl::BGR
    BGRA = 0x80E1,           // gl::BGRA
    RG = 0x8227,             // gl::RG
}

#[luajit_ffi_gen::luajit_ffi]
impl PixelFormat {
    pub fn components(this: Self) -> i32 {
        match this {
            Self::Red | Self::DepthComponent => 1,
            Self::RG => 2,
            Self::RGB | Self::BGR => 3,
            Self::RGBA | Self::BGRA => 4,
        }
    }
}
