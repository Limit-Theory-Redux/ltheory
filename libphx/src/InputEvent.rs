use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::Button::*;
use crate::State::*;
use crate::DeviceType::*;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Button_ToString(_: Button) -> cstr;
    fn Device_ToString(_: *mut Device) -> cstr;
    fn State_ToString(_: State) -> cstr;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: uint32,
}
pub type DeviceType = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: uint32,
    pub device: Device,
    pub button: Button,
    pub value: libc::c_float,
    pub state: State,
}
pub type State = int32;
pub type Button = int32;

#[no_mangle]
pub unsafe extern "C" fn InputEvent_ToString(mut ie: *mut InputEvent) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>())
            as libc::c_int as libc::size_t,
        b"Event %p\n\tTimestamp: %i\n\tDevice:    %s\n\tButton:    %s\n\tValue:     %.2f\n\tState:     %s\0"
            as *const u8 as *const libc::c_char,
        ie,
        (*ie).timestamp,
        Device_ToString(&mut (*ie).device),
        Button_ToString((*ie).button),
        (*ie).value as libc::c_double,
        State_ToString((*ie).state),
    );
    return buffer.as_mut_ptr() as cstr;
}
