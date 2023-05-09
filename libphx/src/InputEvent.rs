use crate::internal::ffi;

use crate::Button::*;

use crate::Device::*;

use crate::State::*;

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
    ffi::StaticString!(format!("Event {:p}\n\tTimestamp: {}\n\tDevice:    {}\n\tButton:    {}\n\tValue:     {:.2}\n\tState:     {}", 
        &*ie,
        (*ie).timestamp,
        ffi::PtrAsSlice(Device_ToString(&mut (*ie).device)),
        ffi::PtrAsSlice(Button_ToString((*ie).button)),
        (*ie).value as f64,
        ffi::PtrAsSlice(State_ToString((*ie).state))
    ))
}
