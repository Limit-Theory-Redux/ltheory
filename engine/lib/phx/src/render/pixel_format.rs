pub type PixelFormat = i32;

#[no_mangle]
pub static PixelFormat_Red: PixelFormat = 1; //gl::RED as PixelFormat;

#[no_mangle]
pub static PixelFormat_RG: PixelFormat = 1; //gl::RG as PixelFormat;

#[no_mangle]
pub static PixelFormat_RGB: PixelFormat = 1; //gl::RGB as PixelFormat;

#[no_mangle]
pub static PixelFormat_BGR: PixelFormat = 1; //gl::BGR as PixelFormat;

#[no_mangle]
pub static PixelFormat_RGBA: PixelFormat = 1; //gl::RGBA as PixelFormat;

#[no_mangle]
pub static PixelFormat_BGRA: PixelFormat = 1; //gl::BGRA as PixelFormat;

#[no_mangle]
pub static PixelFormat_Depth_Component: PixelFormat = 1; //gl::DEPTH_COMPONENT as PixelFormat;

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
