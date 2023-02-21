use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    fn snprintf(_: *mut libc::c_char, _: usize, _: *const libc::c_char, _: ...) -> i32;
    fn Button_ToDeviceType(_: Button) -> DeviceType;
}
pub type cstr = *const libc::c_char;
pub type Button = i32;
pub type DeviceType = i32;

#[no_mangle]
pub unsafe extern "C" fn DeviceType_FromButton(mut button: Button) -> DeviceType {
    return Button_ToDeviceType(button);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceType_ToString(mut deviceType: DeviceType) -> cstr {
    match deviceType {
        0 => return b"DeviceType_Null\0" as *const u8 as *const libc::c_char,
        1 => return b"DeviceType_Mouse\0" as *const u8 as *const libc::c_char,
        2 => return b"DeviceType_Keyboard\0" as *const u8 as *const libc::c_char,
        3 => return b"DeviceType_Gamepad\0" as *const u8 as *const libc::c_char,
        _ => {
            static mut buffer: [libc::c_char; 512] = [0; 512];
            snprintf(
                buffer.as_mut_ptr(),
                (::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(::core::mem::size_of::<libc::c_char>()) as i32
                    as usize,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                deviceType,
            );
            return buffer.as_mut_ptr() as cstr;
        }
    };
}
