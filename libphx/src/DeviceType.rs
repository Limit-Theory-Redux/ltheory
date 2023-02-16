use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Button_ToDeviceType(_: Button) -> DeviceType;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type Button = int32;
pub type DeviceType = int32;

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
                    .wrapping_div(
                        ::core::mem::size_of::<libc::c_char>(),
                    ) as libc::c_int as libc::size_t,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                deviceType,
            );
            return buffer.as_mut_ptr() as cstr;
        }
    };
}
