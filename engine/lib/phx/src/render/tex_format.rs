use super::gl;

#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexFormat {
    R8 = gl::R8,
    R16 = gl::R16,
    R16F = gl::R16F,
    R32F = gl::R32F,
    RG8 = gl::RGB,
    RG16 = gl::RG16,
    RG16F = gl::RG16F,
    RG32F = gl::RG32F,
    RGB8 = gl::RGB8,
    RGBA8 = gl::RGBA8,
    RGBA16 = gl::RGBA16,
    RGBA16F = gl::RGBA16F,
    RGBA32F = gl::RGBA32F,
    Depth16 = gl::DEPTH_COMPONENT16,
    Depth24 = gl::DEPTH_COMPONENT24,
    Depth32F = gl::DEPTH_COMPONENT32F,
}

#[luajit_ffi_gen::luajit_ffi]
impl TexFormat {
    pub fn components(this: Self) -> i32 {
        match this {
            Self::R8
            | Self::R16
            | Self::R16F
            | Self::R32F
            | Self::Depth16
            | Self::Depth24
            | Self::Depth32F => 1,
            Self::RG8 | Self::RG16 | Self::RG16F | Self::RG32F => 2,
            Self::RGB8 => 3,
            Self::RGBA8 | Self::RGBA16 | Self::RGBA16F | Self::RGBA32F => 4,
        }
    }

    pub fn get_size(this: Self) -> i32 {
        match this {
            Self::R8 => 1,
            Self::R16 | Self::R16F | Self::RG8 | Self::Depth16 => 2,
            Self::RGB8 | Self::Depth24 => 3,
            Self::R32F | Self::RG16 | Self::RG16F | Self::RGBA8 | Self::Depth32F => 4,
            Self::RG32F | Self::RGBA16 | Self::RGBA16F => 8,
            Self::RGBA32F => 16,
        }
    }

    pub fn is_color(this: Self) -> bool {
        this != Self::Depth16 && this != Self::Depth24 && this != Self::Depth32F
    }

    pub fn is_depth(this: Self) -> bool {
        this == Self::Depth16 || this == Self::Depth24 || this == Self::Depth32F
    }
}

impl TexFormat {
    /// Returns (internal_format, format, type) for glTexImage2D
    /// Note: Not FFI-exposed due to tuple return type
    pub fn to_gl_formats(self) -> (u32, u32, u32) {
        match self {
            Self::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            Self::R16 => (gl::R16, gl::RED, gl::UNSIGNED_SHORT),
            Self::R16F => (gl::R16F, gl::RED, gl::HALF_FLOAT),
            Self::R32F => (gl::R32F, gl::RED, gl::FLOAT),
            Self::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
            Self::RG16 => (gl::RG16, gl::RG, gl::UNSIGNED_SHORT),
            Self::RG16F => (gl::RG16F, gl::RG, gl::HALF_FLOAT),
            Self::RG32F => (gl::RG32F, gl::RG, gl::FLOAT),
            Self::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            Self::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            Self::RGBA16 => (gl::RGBA16, gl::RGBA, gl::UNSIGNED_SHORT),
            Self::RGBA16F => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
            Self::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
            Self::Depth16 => (gl::DEPTH_COMPONENT16, gl::DEPTH_COMPONENT, gl::UNSIGNED_SHORT),
            Self::Depth24 => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_INT),
            Self::Depth32F => (gl::DEPTH_COMPONENT32F, gl::DEPTH_COMPONENT, gl::FLOAT),
        }
    }
}
