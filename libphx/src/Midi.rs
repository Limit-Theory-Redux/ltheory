use crate::internal::Memory::*;
use glam::IVec2;
use glam::Vec3;
use libc;
extern "C" {
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MidiDevice {
    pub cursor: i32,
    pub buffer: [IVec2; 512],
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetCount() -> i32 {
    return 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Open(mut index: i32) -> *mut MidiDevice {
    return 0 as *mut MidiDevice;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Close(mut this: *mut MidiDevice) {}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetNameByIndex(mut index: i32) -> cstr {
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_HasMessage(mut this: *mut MidiDevice) -> bool {
    return (*this).cursor > 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_PopMessage(mut this: *mut MidiDevice) -> IVec2 {
    if (*this).cursor <= 0 as i32 {
        Fatal(
            b"MidiDevice_PopMessage: device has no messages\0" as *const u8 as *const libc::c_char,
        );
    }
    (*this).cursor -= 1 as i32;
    return (*this).buffer[(*this).cursor as usize];
}
