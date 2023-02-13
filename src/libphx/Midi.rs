use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MidiDevice {
    pub cursor: libc::c_int,
    pub buffer: [Vec2i; 512],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
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
pub unsafe extern "C" fn MidiDevice_Close(mut self_0: *mut MidiDevice) {}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_GetNameByIndex(mut index: libc::c_int) -> cstr {
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_HasMessage(mut self_0: *mut MidiDevice) -> bool {
    return (*self_0).cursor > 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MidiDevice_PopMessage(mut self_0: *mut MidiDevice) -> Vec2i {
    if (*self_0).cursor <= 0 as libc::c_int {
        Fatal(
            b"MidiDevice_PopMessage: device has no messages\0" as *const u8
                as *const libc::c_char,
        );
    }
    (*self_0).cursor -= 1 as libc::c_int;
    return (*self_0).buffer[(*self_0).cursor as usize];
}
