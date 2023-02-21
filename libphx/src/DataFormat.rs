use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type DataFormat = i32;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0x1401 as i32;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0x1400 as i32;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0x1403 as i32;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0x1402 as i32;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0x1405 as i32;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0x1404 as i32;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0x1406 as i32;
#[no_mangle]
pub unsafe extern "C" fn DataFormat_GetSize(mut this: DataFormat) -> i32 {
    match this {
        5121 | 5120 => return 1 as i32,
        5123 | 5122 => return 2 as i32,
        5125 | 5124 | 5126 => return 4 as i32,
        _ => {}
    }
    return 0 as i32;
}
