use crate::gl::gl;
pub type TexFormat = i32;

#[no_mangle]
pub static TexFormat_R8: TexFormat = gl::R8 as TexFormat;

#[no_mangle]
pub static TexFormat_R16: TexFormat = gl::R16 as TexFormat;

#[no_mangle]
pub static TexFormat_R16F: TexFormat = gl::R16F as TexFormat;

#[no_mangle]
pub static TexFormat_R32F: TexFormat = gl::R32F as TexFormat;

#[no_mangle]
pub static TexFormat_RG8: TexFormat = gl::RGB as TexFormat;

#[no_mangle]
pub static TexFormat_RG16: TexFormat = gl::RG16 as TexFormat;

#[no_mangle]
pub static TexFormat_RG16F: TexFormat = gl::RG16F as TexFormat;

#[no_mangle]
pub static TexFormat_RG32F: TexFormat = gl::RG32F as TexFormat;

#[no_mangle]
pub static TexFormat_RGB8: TexFormat = gl::RGB8 as TexFormat;

#[no_mangle]
pub static TexFormat_RGBA8: TexFormat = gl::RGBA8 as TexFormat;

#[no_mangle]
pub static TexFormat_RGBA16: TexFormat = gl::RGBA16 as TexFormat;

// #[no_mangle]
// pub static TexFormat_RGBA16F: TexFormat = gl::RGBA16F as TexFormat;

#[no_mangle]
pub static TexFormat_RGBA16F: TexFormat = 0x881a;

// #[no_mangle]
// pub static TexFormat_RGBA32F: TexFormat = gl::RGBA32F as TexFormat;
#[no_mangle]
pub static TexFormat_RGBA32F: TexFormat = 0x8814;

#[no_mangle]
pub static TexFormat_Depth16: TexFormat = gl::DEPTH_COMPONENT16 as TexFormat;

#[no_mangle]
pub static TexFormat_Depth24: TexFormat = gl::DEPTH_COMPONENT24 as TexFormat;

// #[no_mangle]
// pub static TexFormat_Depth32F: TexFormat = gl::DEPTH_COMPONENT32F as TexFormat;
#[no_mangle]
pub static TexFormat_Depth32F: TexFormat = 0x8cac;

#[no_mangle]
pub extern "C" fn TexFormat_Components(this: TexFormat) -> i32 {
    if this == TexFormat_R8
        || this == TexFormat_R16
        || this == TexFormat_R16F
        || this == TexFormat_R32F
        || this == TexFormat_Depth16
        || this == TexFormat_Depth24
        || this == TexFormat_Depth32F
    {
        1
    } else if this == TexFormat_RG8
        || this == TexFormat_RG16
        || this == TexFormat_RG16F
        || this == TexFormat_RG32F
    {
        2
    } else if this == TexFormat_RGB8 {
        3
    } else if this == TexFormat_RGBA8
        || this == TexFormat_RGBA16
        || this == TexFormat_RGBA16F
        || this == TexFormat_RGBA32F
    {
        4
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn TexFormat_GetSize(this: TexFormat) -> i32 {
    if this == TexFormat_R8 {
        1
    } else if this == TexFormat_R16
        || this == TexFormat_R16F
        || this == TexFormat_RG8
        || this == TexFormat_Depth16
    {
        2
    } else if this == TexFormat_RGB8 || this == TexFormat_Depth24 {
        3
    } else if this == TexFormat_R32F
        || this == TexFormat_RG16
        || this == TexFormat_RG16F
        || this == TexFormat_RGBA8
        || this == TexFormat_Depth32F
    {
        4
    } else if this == TexFormat_RG32F || this == TexFormat_RGBA16 || this == TexFormat_RGBA16F {
        8
    } else if this == TexFormat_RGBA32F {
        16
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn TexFormat_IsColor(this: TexFormat) -> bool {
    this != TexFormat_Depth16 && this != TexFormat_Depth24 && this != TexFormat_Depth32F
}

#[no_mangle]
pub extern "C" fn TexFormat_IsDepth(this: TexFormat) -> bool {
    !TexFormat_IsColor(this)
}

#[no_mangle]
pub extern "C" fn TexFormat_IsValid(this: TexFormat) -> bool {
    this == TexFormat_R8
        || this == TexFormat_R16
        || this == TexFormat_R16F
        || this == TexFormat_R32F
        || this == TexFormat_RG8
        || this == TexFormat_RG16
        || this == TexFormat_RG16F
        || this == TexFormat_RG32F
        || this == TexFormat_RGB8
        || this == TexFormat_RGBA8
        || this == TexFormat_RGBA16
        || this == TexFormat_RGBA16F
        || this == TexFormat_RGBA32F
        || this == TexFormat_Depth16
        || this == TexFormat_Depth24
        || this == TexFormat_Depth32F
}
