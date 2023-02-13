use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::c_ulong,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Button_ToDeviceType(_: Button) -> DeviceType;
}
pub type int32_t = libc::c_int;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type Button = int32;
pub type DeviceType = int32;
#[no_mangle]
pub static mut Button_Null: Button = 0;
#[no_mangle]
pub static mut Button_First: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_First: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_A: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_B: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_C: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_D: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_E: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_G: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_H: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_I: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_J: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_K: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_L: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_M: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_O: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_P: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Q: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_R: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_S: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_T: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_U: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_V: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_W: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_X: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Y: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Z: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N0: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F10: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F11: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F12: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F13: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F14: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F15: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F16: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F17: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F18: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F19: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F20: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F21: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F22: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F23: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F24: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP0: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPNumLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPDivide: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPMultiply: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPSubtract: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPAdd: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPEnter: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPDecimal: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backspace: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Escape: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Return: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Space: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Tab: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backtick: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_CapsLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Minus: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Equals: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LBracket: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RBracket: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backslash: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Semicolon: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Apostrophe: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Comma: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Period: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Slash: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PrintScreen: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_ScrollLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Pause: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Insert: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Delete: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Home: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_End: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PageUp: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PageDown: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Right: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Left: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Down: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Up: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LCtrl: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LShift: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LAlt: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LMeta: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RCtrl: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RShift: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RAlt: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RMeta: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Last: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_First: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Left: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Middle: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Right: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X1: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X2: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Y: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_ScrollX: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_ScrollY: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Button_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_A: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_B: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_X: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Y: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Back: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Guide: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Start: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStick: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStick: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LBumper: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RBumper: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Up: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Down: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Left: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Right: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Button_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Axis_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LTrigger: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RTrigger: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStickX: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStickY: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStickX: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStickY: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Axis_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Last: Button = 0;
#[no_mangle]
pub static mut Button_System_First: Button = 0;
#[no_mangle]
pub static mut Button_System_Exit: Button = 0;
#[no_mangle]
pub static mut Button_System_Last: Button = 0;
#[no_mangle]
pub static mut Button_Last: Button = 0;
#[no_mangle]
pub static mut DeviceType_Null: DeviceType = 0 as libc::c_int;
#[no_mangle]
pub static mut DeviceType_Mouse: DeviceType = 1 as libc::c_int;
#[no_mangle]
pub static mut DeviceType_Keyboard: DeviceType = 2 as libc::c_int;
#[no_mangle]
pub static mut DeviceType_Gamepad: DeviceType = 3 as libc::c_int;
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
                    ) as libc::c_int as size_t,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                deviceType,
            );
            return buffer.as_mut_ptr() as cstr;
        }
    };
}
