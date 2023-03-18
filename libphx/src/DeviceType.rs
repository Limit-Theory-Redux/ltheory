use crate::internal::Memory::*;
use crate::Common::*;
use crate::Button::*;
use crate::Math::Vec3;
use libc;

pub type Button = i32;
pub type DeviceType = i32;

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
                (::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(::core::mem::size_of::<libc::c_char>()) as i32
                    as usize,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                deviceType,
            );
            buffer.as_mut_ptr() as *const libc::c_char
        }
    }
}
