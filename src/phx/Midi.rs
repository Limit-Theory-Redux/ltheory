
use crate::phx::Common::*;
use crate::phx::Math::IVec2;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct MidiDevice {
    pub cursor: i32,
    pub buffer: [IVec2; 512],
}

#[no_mangle]
pub extern "C" fn MidiDevice_GetCount() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn MidiDevice_Open(_index: i32) -> *mut MidiDevice {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn MidiDevice_Close(_this: &mut MidiDevice) {}

#[no_mangle]
pub extern "C" fn MidiDevice_GetNameByIndex(_index: i32) -> *const libc::c_char {
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn MidiDevice_HasMessage(this: &mut MidiDevice) -> bool {
    this.cursor > 0
}

#[no_mangle]
pub unsafe extern "C" fn MidiDevice_PopMessage(this: &mut MidiDevice) -> IVec2 {
    if this.cursor <= 0 {
        CFatal!("MidiDevice_PopMessage: device has no messages");
    }
    this.cursor -= 1;
    this.buffer[this.cursor as usize]
}
