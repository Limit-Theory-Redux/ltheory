use crate::button::*;
use crate::device::*;
use crate::state::*;
use crate::static_string;
use crate::Convert;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: u32,
    pub device: Device,
    pub button: Button,
    pub value: f32,
    pub state: State,
}

impl InputEvent {
    pub fn to_string(&self) -> String {
        format!("Event {:p}\n\tTimestamp: {}\n\tDevice:    {}\n\tButton:    {}\n\tValue:     {:.2}\n\tState:     {}",
            &*self,
            (*self).timestamp,
            (*self).device.to_string(),
            button_to_string((*self).button),
            (*self).value as f64,
            state_to_string((*self).state)
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn InputEvent_ToString(ie: *mut InputEvent) -> *const libc::c_char {
    static_string!((*ie).to_string())
}
