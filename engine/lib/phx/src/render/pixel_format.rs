use super::gl;

#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    DepthComponent = gl::DEPTH_COMPONENT,
    Red = gl::RED,
    RGB = gl::RGB,
    RGBA = gl::RGBA,
    BGR = gl::BGR,
    BGRA = gl::BGRA,
    RG = gl::RG,
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

impl PixelFormat {
    /// The layout with `n` components (1 to 4), for a payload described by a
    /// `TexFormat`.
    pub fn for_components(n: i32) -> Self {
        match n {
            1 => Self::Red,
            2 => Self::RG,
            3 => Self::RGB,
            _ => Self::RGBA,
        }
    }
}
