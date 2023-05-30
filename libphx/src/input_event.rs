use crate::button::*;
use crate::Convert;

use crate::device::*;

use crate::state::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: u32,
    pub device: Device,
    pub button: Button,
    pub value: f32,
    pub state: State,
}

#[no_mangle]
pub unsafe extern "C" fn InputEvent_ToString(ie: *mut InputEvent) -> *const libc::c_char {
    format!("Event {:p}\n\tTimestamp: {}\n\tDevice:    {}\n\tButton:    {}\n\tValue:     {:.2}\n\tState:     {}",
        &*ie,
        (*ie).timestamp,
        (*ie).device.to_string(),
        button_to_string((*ie).button),
        (*ie).value as f64,
        state_to_string((*ie).state)
    ).convert()
}
