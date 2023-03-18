use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type DataFormat = i32;

#[no_mangle]
pub static DataFormat_U8: DataFormat = 0x1401_i32;

#[no_mangle]
pub static DataFormat_I8: DataFormat = 0x1400_i32;

#[no_mangle]
pub static DataFormat_U16: DataFormat = 0x1403_i32;

#[no_mangle]
pub static DataFormat_I16: DataFormat = 0x1402_i32;

#[no_mangle]
pub static DataFormat_U32: DataFormat = 0x1405_i32;

#[no_mangle]
pub static DataFormat_I32: DataFormat = 0x1404_i32;

#[no_mangle]
pub static DataFormat_Float: DataFormat = 0x1406_i32;

#[no_mangle]
pub unsafe extern "C" fn DataFormat_GetSize(mut this: DataFormat) -> i32 {
    match this {
        5121 | 5120 => return 1_i32,
        5123 | 5122 => return 2_i32,
        5125 | 5124 | 5126 => return 4_i32,
        _ => {}
    }
    return 0_i32;
}
