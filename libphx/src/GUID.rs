use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type uint64_t = libc::c_ulonglong;
pub type uint64 = uint64_t;
#[no_mangle]
pub static mut nextID: uint64 = 1 as libc::c_int as uint64;
#[no_mangle]
pub unsafe extern "C" fn GUID_Create() -> uint64 {
    let fresh0 = nextID;
    nextID = nextID.wrapping_add(1);
    return fresh0;
}
#[no_mangle]
pub unsafe extern "C" fn GUID_Exists(mut id: uint64) -> bool {
    return id < nextID && id != 0 as libc::c_ulonglong;
}
#[no_mangle]
pub unsafe extern "C" fn GUID_Reset() {
    nextID = 1 as libc::c_int as uint64;
}
