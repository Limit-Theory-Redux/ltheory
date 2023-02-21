use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::Button::*;
use crate::State::*;
use crate::DeviceType::*;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: usize,
        _: *const libc::c_char,
        _: ...
    ) -> i32;
    fn Button_ToString(_: Button) -> cstr;
    fn Device_ToString(_: *mut Device) -> cstr;
    fn State_ToString(_: State) -> cstr;
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: u32,
}
pub type DeviceType = i32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: u32,
    pub device: Device,
    pub button: Button,
    pub value: f32,
    pub state: State,
}
pub type State = i32;
pub type Button = i32;

#[no_mangle]
pub unsafe extern "C" fn InputEvent_ToString(mut ie: *mut InputEvent) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>())
            as i32 as usize,
        b"Event %p\n\tTimestamp: %i\n\tDevice:    %s\n\tButton:    %s\n\tValue:     %.2f\n\tState:     %s\0"
            as *const u8 as *const libc::c_char,
        ie,
        (*ie).timestamp,
        Device_ToString(&mut (*ie).device),
        Button_ToString((*ie).button),
        (*ie).value as f64,
        State_ToString((*ie).state),
    );
    return buffer.as_mut_ptr() as cstr;
}
