use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type PixelFormat = i32;
#[no_mangle]
pub static mut PixelFormat_Red: PixelFormat = 0x1903 as i32;
#[no_mangle]
pub static mut PixelFormat_RG: PixelFormat = 0x8227 as i32;
#[no_mangle]
pub static mut PixelFormat_RGB: PixelFormat = 0x1907 as i32;
#[no_mangle]
pub static mut PixelFormat_BGR: PixelFormat = 0x80e0 as i32;
#[no_mangle]
pub static mut PixelFormat_RGBA: PixelFormat = 0x1908 as i32;
#[no_mangle]
pub static mut PixelFormat_BGRA: PixelFormat = 0x80e1 as i32;
#[no_mangle]
pub static mut PixelFormat_Depth_Component: PixelFormat = 0x1902 as i32;
#[no_mangle]
pub unsafe extern "C" fn PixelFormat_Components(mut this: PixelFormat) -> i32 {
    match this {
        6403 | 6402 => return 1 as i32,
        33319 => return 2 as i32,
        6407 | 32992 => return 3 as i32,
        6408 | 32993 => return 4 as i32,
        _ => {}
    }
    return 0 as i32;
}
