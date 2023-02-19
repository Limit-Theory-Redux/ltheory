use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type DataFormat = int32;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0x1401 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0x1400 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0x1403 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0x1402 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0x1405 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0x1404 as libc::c_int;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0x1406 as libc::c_int;
#[no_mangle]
pub unsafe extern "C" fn DataFormat_GetSize(mut self_0: DataFormat) -> libc::c_int {
    match self_0 {
        5121 | 5120 => return 1 as libc::c_int,
        5123 | 5122 => return 2 as libc::c_int,
        5125 | 5124 | 5126 => return 4 as libc::c_int,
        _ => {}
    }
    return 0 as libc::c_int;
}
