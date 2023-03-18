use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
pub type PixelFormat = i32;

#[no_mangle]
pub static PixelFormat_Red: PixelFormat = 0x1903_i32;

#[no_mangle]
pub static PixelFormat_RG: PixelFormat = 0x8227_i32;

#[no_mangle]
pub static PixelFormat_RGB: PixelFormat = 0x1907_i32;

#[no_mangle]
pub static PixelFormat_BGR: PixelFormat = 0x80e0_i32;

#[no_mangle]
pub static PixelFormat_RGBA: PixelFormat = 0x1908_i32;

#[no_mangle]
pub static PixelFormat_BGRA: PixelFormat = 0x80e1_i32;

#[no_mangle]
pub static PixelFormat_Depth_Component: PixelFormat = 0x1902_i32;

#[no_mangle]
pub unsafe extern "C" fn PixelFormat_Components(mut this: PixelFormat) -> i32 {
    match this {
        6403 | 6402 => return 1_i32,
        33319 => return 2_i32,
        6407 | 32992 => return 3_i32,
        6408 | 32993 => return 4_i32,
        _ => {}
    }
    0_i32
}
