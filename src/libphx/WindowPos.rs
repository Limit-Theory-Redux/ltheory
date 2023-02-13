use ::libc;
use crate::internal::Memory::*;
pub type WindowPos = libc::c_int;
#[no_mangle]
pub static mut WindowPos_Centered: WindowPos = (0x2fff0000 as libc::c_uint
    | 0 as libc::c_int as libc::c_uint) as WindowPos;
#[no_mangle]
pub static mut WindowPos_Default: WindowPos = (0x1fff0000 as libc::c_uint
    | 0 as libc::c_int as libc::c_uint) as WindowPos;
