use super::*;
use crate::common::*;
use crate::internal::*;

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
pub extern "C" fn DeviceType_ToString(deviceType: DeviceType) -> *const libc::c_char {
    static_string!(device_type_to_string(deviceType))
}

pub fn device_type_to_string(deviceType: DeviceType) -> String {
    match deviceType {
        dt if dt == DeviceType_Null => "DeviceType_Null",
        dt if dt == DeviceType_Mouse => "DeviceType_Mouse",
        dt if dt == DeviceType_Keyboard => "DeviceType_Keyboard",
        dt if dt == DeviceType_Gamepad => "DeviceType_Gamepad",
        _ => return format!("Unknown ({})", deviceType),
    }
    .into()
}
