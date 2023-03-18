use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

#[no_mangle]
pub static mut nextID: u64 = 1_i32 as u64;

#[no_mangle]
pub unsafe extern "C" fn GUID_Create() -> u64 {
    let fresh0 = nextID;
    nextID = nextID.wrapping_add(1);
    fresh0
}

#[no_mangle]
pub unsafe extern "C" fn GUID_Exists(mut id: u64) -> bool {
    id < nextID && id != 0_u64
}

#[no_mangle]
pub unsafe extern "C" fn GUID_Reset() {
    nextID = 1_i32 as u64;
}
