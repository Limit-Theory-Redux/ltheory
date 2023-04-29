use crate::internal::ffi;
use crate::internal::Memory::*;
use crate::Button::*;
use crate::Common::*;
use crate::Math::Vec3;

pub type DeviceType = i32;

#[no_mangle]
pub static DeviceType_Null: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Mouse: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Keyboard: DeviceType = 0;

#[no_mangle]
pub static DeviceType_Gamepad: DeviceType = 0;

pub const DeviceType_COUNT: usize = 4;

#[no_mangle]
pub extern "C" fn DeviceType_FromButton(button: Button) -> DeviceType {
    Button_ToDeviceType(button)
}

#[no_mangle]
pub unsafe extern "C" fn DeviceType_ToString(deviceType: DeviceType) -> *const libc::c_char {
    match deviceType {
        dt if dt == DeviceType_Null => c_str!("DeviceType_Null"),
        dt if dt == DeviceType_Mouse => c_str!("DeviceType_Mouse"),
        dt if dt == DeviceType_Keyboard => c_str!("DeviceType_Keyboard"),
        dt if dt == DeviceType_Gamepad => c_str!("DeviceType_Gamepad"),
        _ => {
            ffi::StaticString!(format!("Unknown ({})", deviceType))
        }
    }
}
