use crate::internal::Memory::*;
use crate::Common::*;
use crate::GL::gl;
pub type PixelFormat = i32;

#[no_mangle]
pub static PixelFormat_Red: PixelFormat = gl::RED as PixelFormat;

#[no_mangle]
pub static PixelFormat_RG: PixelFormat = gl::RG as PixelFormat;

#[no_mangle]
pub static PixelFormat_RGB: PixelFormat = gl::RGB as PixelFormat;

#[no_mangle]
pub static PixelFormat_BGR: PixelFormat = gl::BGR as PixelFormat;

#[no_mangle]
pub static PixelFormat_RGBA: PixelFormat = gl::RGBA as PixelFormat;

#[no_mangle]
pub static PixelFormat_BGRA: PixelFormat = gl::BGRA as PixelFormat;

#[no_mangle]
pub static PixelFormat_Depth_Component: PixelFormat = gl::DEPTH_COMPONENT as PixelFormat;

#[no_mangle]
pub extern "C" fn PixelFormat_Components(this: PixelFormat) -> i32 {
    if this == PixelFormat_Red || this == PixelFormat_Depth_Component {
        1
    } else if this == PixelFormat_RG {
        2
    } else if this == PixelFormat_RGB || this == PixelFormat_BGR {
        3
    } else if this == PixelFormat_RGBA || this == PixelFormat_BGRA {
        4
    } else {
        0
    }
}
