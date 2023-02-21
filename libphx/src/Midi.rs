use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MidiDevice {
    pub cursor: libc::c_int,
    pub buffer: [IVec2; 512],
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetCount() -> libc::c_int {
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Open(mut index: libc::c_int) -> *mut MidiDevice {
    return 0 as *mut MidiDevice;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_Close(mut this: *mut MidiDevice) {}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetNameByIndex(mut index: libc::c_int) -> cstr {
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_HasMessage(mut this: *mut MidiDevice) -> bool {
    return (*this).cursor > 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_PopMessage(mut this: *mut MidiDevice) -> IVec2 {
    if (*this).cursor <= 0 as libc::c_int {
        Fatal(
            b"MidiDevice_PopMessage: device has no messages\0" as *const u8
                as *const libc::c_char,
        );
    }
    (*this).cursor -= 1 as libc::c_int;
    return (*this).buffer[(*this).cursor as usize];
}
