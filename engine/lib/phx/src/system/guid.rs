#[no_mangle]
pub static mut nextID: u64 = 1;

#[no_mangle]
pub unsafe extern "C" fn GUID_Create() -> u64 {
    let fresh0 = nextID;
    nextID = nextID.wrapping_add(1);
    fresh0
}

#[no_mangle]
pub unsafe extern "C" fn GUID_Exists(id: u64) -> bool {
    id < nextID && id != 0
}

#[no_mangle]
pub unsafe extern "C" fn GUID_Reset() {
    nextID = 1;
}
