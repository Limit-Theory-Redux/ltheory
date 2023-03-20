use crate::internal::Memory::*;
use crate::Button::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type DeviceType = i32;

#[no_mangle]
pub static DeviceType_Null: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Mouse: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Keyboard: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Gamepad: DeviceType = 0;

#[no_mangle]
pub unsafe extern "C" fn DeviceType_FromButton(mut button: Button) -> DeviceType {
    Button_ToDeviceType(button)
}

#[no_mangle]
pub unsafe extern "C" fn DeviceType_ToString(mut deviceType: DeviceType) -> *const libc::c_char {
    match deviceType {
        0 => b"DeviceType_Null\0" as *const u8 as *const libc::c_char,
        1 => b"DeviceType_Mouse\0" as *const u8 as *const libc::c_char,
        2 => b"DeviceType_Keyboard\0" as *const u8 as *const libc::c_char,
        3 => b"DeviceType_Gamepad\0" as *const u8 as *const libc::c_char,
        _ => {
            static mut buffer: [libc::c_char; 512] = [0; 512];
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                deviceType,
            );
            buffer.as_mut_ptr() as *const libc::c_char
        }
    }
}
