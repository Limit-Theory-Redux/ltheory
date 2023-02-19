use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;

pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;

#[no_mangle]
pub unsafe extern "C" fn Bit_And32(mut x: uint32, mut y: uint32) -> uint32 {
    return x & y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Or32(mut x: uint32, mut y: uint32) -> uint32 {
    return x | y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Xor32(mut x: uint32, mut y: uint32) -> uint32 {
    return x ^ y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Has32(mut x: uint32, mut y: uint32) -> bool {
    return x & y == y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_And64(mut x: uint64, mut y: uint64) -> uint64 {
    return x & y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Or64(mut x: uint64, mut y: uint64) -> uint64 {
    return x | y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Xor64(mut x: uint64, mut y: uint64) -> uint64 {
    return x ^ y;
}
#[no_mangle]
pub unsafe extern "C" fn Bit_Has64(mut x: uint64, mut y: uint64) -> bool {
    return x & y == y;
}
