use crate::internal::Memory::*;
use glam::IVec2;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MidiDevice {
    pub cursor: i32,
    pub buffer: [IVec2; 512],
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetCount() -> i32 {
    0_i32
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Open(mut _index: i32) -> *mut MidiDevice {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Close(mut _this: *mut MidiDevice) {}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetNameByIndex(mut _index: i32) -> *const libc::c_char {
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_HasMessage(mut this: *mut MidiDevice) -> bool {
    (*this).cursor > 0_i32
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_PopMessage(mut this: *mut MidiDevice) -> IVec2 {
    if (*this).cursor <= 0_i32 {
        Fatal(
            b"MidiDevice_PopMessage: device has no messages\0" as *const u8 as *const libc::c_char,
        );
    }
    (*this).cursor -= 1_i32;
    (*this).buffer[(*this).cursor as usize]
}