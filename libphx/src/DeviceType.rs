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
pub unsafe extern "C" fn DeviceType_FromButton(button: Button) -> DeviceType {
    Button_ToDeviceType(button)
}

#[no_mangle]
pub unsafe extern "C" fn DeviceType_ToString(deviceType: DeviceType) -> *const libc::c_char {
    match deviceType {
        0 => c_str!("DeviceType_Null"),
        1 => c_str!("DeviceType_Mouse"),
        2 => c_str!("DeviceType_Keyboard"),
        3 => c_str!("DeviceType_Gamepad"),
        _ => {
            static mut buffer: [libc::c_char; 512] = [0; 512];
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                c_str!("Unknown (%i)"),
                deviceType,
            );
            buffer.as_mut_ptr() as *const libc::c_char
        }
    }
}
