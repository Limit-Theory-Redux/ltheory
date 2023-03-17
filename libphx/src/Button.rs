use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
}
pub type Button = i32;
pub type DeviceType = i32;

#[no_mangle]
pub static mut DeviceType_Null: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Mouse: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Keyboard: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Gamepad: DeviceType = 0;
#[no_mangle]
pub static mut Button_Null: Button = 0 as i32;
#[no_mangle]
pub static mut Button_First: Button = 1 as i32;
#[no_mangle]
pub static mut Button_Keyboard_First: Button = unsafe { Button_First };
#[no_mangle]
pub static mut Button_Keyboard_A: Button = unsafe { Button_Keyboard_First + 0 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_B: Button = unsafe { Button_Keyboard_First + 1 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_C: Button = unsafe { Button_Keyboard_First + 2 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_D: Button = unsafe { Button_Keyboard_First + 3 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_E: Button = unsafe { Button_Keyboard_First + 4 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F: Button = unsafe { Button_Keyboard_First + 5 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_G: Button = unsafe { Button_Keyboard_First + 6 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_H: Button = unsafe { Button_Keyboard_First + 7 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_I: Button = unsafe { Button_Keyboard_First + 8 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_J: Button = unsafe { Button_Keyboard_First + 9 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_K: Button = unsafe { Button_Keyboard_First + 10 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_L: Button = unsafe { Button_Keyboard_First + 11 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_M: Button = unsafe { Button_Keyboard_First + 12 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N: Button = unsafe { Button_Keyboard_First + 13 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_O: Button = unsafe { Button_Keyboard_First + 14 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_P: Button = unsafe { Button_Keyboard_First + 15 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Q: Button = unsafe { Button_Keyboard_First + 16 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_R: Button = unsafe { Button_Keyboard_First + 17 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_S: Button = unsafe { Button_Keyboard_First + 18 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_T: Button = unsafe { Button_Keyboard_First + 19 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_U: Button = unsafe { Button_Keyboard_First + 20 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_V: Button = unsafe { Button_Keyboard_First + 21 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_W: Button = unsafe { Button_Keyboard_First + 22 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_X: Button = unsafe { Button_Keyboard_First + 23 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Y: Button = unsafe { Button_Keyboard_First + 24 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Z: Button = unsafe { Button_Keyboard_First + 25 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N0: Button = unsafe { Button_Keyboard_First + 26 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N1: Button = unsafe { Button_Keyboard_First + 27 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N2: Button = unsafe { Button_Keyboard_First + 28 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N3: Button = unsafe { Button_Keyboard_First + 29 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N4: Button = unsafe { Button_Keyboard_First + 30 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N5: Button = unsafe { Button_Keyboard_First + 31 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N6: Button = unsafe { Button_Keyboard_First + 32 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N7: Button = unsafe { Button_Keyboard_First + 33 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N8: Button = unsafe { Button_Keyboard_First + 34 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_N9: Button = unsafe { Button_Keyboard_First + 35 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F1: Button = unsafe { Button_Keyboard_First + 36 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F2: Button = unsafe { Button_Keyboard_First + 37 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F3: Button = unsafe { Button_Keyboard_First + 38 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F4: Button = unsafe { Button_Keyboard_First + 39 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F5: Button = unsafe { Button_Keyboard_First + 40 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F6: Button = unsafe { Button_Keyboard_First + 41 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F7: Button = unsafe { Button_Keyboard_First + 42 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F8: Button = unsafe { Button_Keyboard_First + 43 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F9: Button = unsafe { Button_Keyboard_First + 44 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F10: Button = unsafe { Button_Keyboard_First + 45 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F11: Button = unsafe { Button_Keyboard_First + 46 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F12: Button = unsafe { Button_Keyboard_First + 47 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F13: Button = unsafe { Button_Keyboard_First + 48 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F14: Button = unsafe { Button_Keyboard_First + 49 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F15: Button = unsafe { Button_Keyboard_First + 50 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F16: Button = unsafe { Button_Keyboard_First + 51 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F17: Button = unsafe { Button_Keyboard_First + 52 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F18: Button = unsafe { Button_Keyboard_First + 53 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F19: Button = unsafe { Button_Keyboard_First + 54 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F20: Button = unsafe { Button_Keyboard_First + 55 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F21: Button = unsafe { Button_Keyboard_First + 56 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F22: Button = unsafe { Button_Keyboard_First + 57 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F23: Button = unsafe { Button_Keyboard_First + 58 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_F24: Button = unsafe { Button_Keyboard_First + 59 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP0: Button = unsafe { Button_Keyboard_First + 60 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP1: Button = unsafe { Button_Keyboard_First + 61 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP2: Button = unsafe { Button_Keyboard_First + 62 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP3: Button = unsafe { Button_Keyboard_First + 63 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP4: Button = unsafe { Button_Keyboard_First + 64 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP5: Button = unsafe { Button_Keyboard_First + 65 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP6: Button = unsafe { Button_Keyboard_First + 66 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP7: Button = unsafe { Button_Keyboard_First + 67 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP8: Button = unsafe { Button_Keyboard_First + 68 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KP9: Button = unsafe { Button_Keyboard_First + 69 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPNumLock: Button = unsafe { Button_Keyboard_First + 70 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPDivide: Button = unsafe { Button_Keyboard_First + 71 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPMultiply: Button = unsafe { Button_Keyboard_First + 72 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPSubtract: Button = unsafe { Button_Keyboard_First + 73 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPAdd: Button = unsafe { Button_Keyboard_First + 74 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPEnter: Button = unsafe { Button_Keyboard_First + 75 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_KPDecimal: Button = unsafe { Button_Keyboard_First + 76 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Backspace: Button = unsafe { Button_Keyboard_First + 77 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Escape: Button = unsafe { Button_Keyboard_First + 78 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Return: Button = unsafe { Button_Keyboard_First + 79 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Space: Button = unsafe { Button_Keyboard_First + 80 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Tab: Button = unsafe { Button_Keyboard_First + 81 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Backtick: Button = unsafe { Button_Keyboard_First + 82 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_CapsLock: Button = unsafe { Button_Keyboard_First + 83 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Minus: Button = unsafe { Button_Keyboard_First + 84 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Equals: Button = unsafe { Button_Keyboard_First + 85 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_LBracket: Button = unsafe { Button_Keyboard_First + 86 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_RBracket: Button = unsafe { Button_Keyboard_First + 87 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Backslash: Button = unsafe { Button_Keyboard_First + 88 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Semicolon: Button = unsafe { Button_Keyboard_First + 89 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Apostrophe: Button = unsafe { Button_Keyboard_First + 90 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Comma: Button = unsafe { Button_Keyboard_First + 91 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Period: Button = unsafe { Button_Keyboard_First + 92 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Slash: Button = unsafe { Button_Keyboard_First + 93 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_PrintScreen: Button = unsafe { Button_Keyboard_First + 94 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_ScrollLock: Button = unsafe { Button_Keyboard_First + 95 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Pause: Button = unsafe { Button_Keyboard_First + 96 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Insert: Button = unsafe { Button_Keyboard_First + 97 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Delete: Button = unsafe { Button_Keyboard_First + 98 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Home: Button = unsafe { Button_Keyboard_First + 99 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_End: Button = unsafe { Button_Keyboard_First + 100 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_PageUp: Button = unsafe { Button_Keyboard_First + 101 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_PageDown: Button = unsafe { Button_Keyboard_First + 102 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Right: Button = unsafe { Button_Keyboard_First + 103 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Left: Button = unsafe { Button_Keyboard_First + 104 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Down: Button = unsafe { Button_Keyboard_First + 105 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Up: Button = unsafe { Button_Keyboard_First + 106 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_LCtrl: Button = unsafe { Button_Keyboard_First + 107 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_LShift: Button = unsafe { Button_Keyboard_First + 108 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_LAlt: Button = unsafe { Button_Keyboard_First + 109 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_LMeta: Button = unsafe { Button_Keyboard_First + 110 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_RCtrl: Button = unsafe { Button_Keyboard_First + 111 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_RShift: Button = unsafe { Button_Keyboard_First + 112 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_RAlt: Button = unsafe { Button_Keyboard_First + 113 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_RMeta: Button = unsafe { Button_Keyboard_First + 114 as i32 };
#[no_mangle]
pub static mut Button_Keyboard_Last: Button = unsafe { Button_Keyboard_First + 115 as i32 };
#[no_mangle]
pub static mut Button_Mouse_First: Button = unsafe { Button_Keyboard_Last + 1 as i32 };
#[no_mangle]
pub static mut Button_Mouse_Left: Button = unsafe { Button_Mouse_First + 0 as i32 };
#[no_mangle]
pub static mut Button_Mouse_Middle: Button = unsafe { Button_Mouse_First + 1 as i32 };
#[no_mangle]
pub static mut Button_Mouse_Right: Button = unsafe { Button_Mouse_First + 2 as i32 };
#[no_mangle]
pub static mut Button_Mouse_X1: Button = unsafe { Button_Mouse_First + 3 as i32 };
#[no_mangle]
pub static mut Button_Mouse_X2: Button = unsafe { Button_Mouse_First + 4 as i32 };
#[no_mangle]
pub static mut Button_Mouse_X: Button = unsafe { Button_Mouse_First + 5 as i32 };
#[no_mangle]
pub static mut Button_Mouse_Y: Button = unsafe { Button_Mouse_First + 6 as i32 };
#[no_mangle]
pub static mut Button_Mouse_ScrollX: Button = unsafe { Button_Mouse_First + 7 as i32 };
#[no_mangle]
pub static mut Button_Mouse_ScrollY: Button = unsafe { Button_Mouse_First + 8 as i32 };
#[no_mangle]
pub static mut Button_Mouse_Last: Button = unsafe { Button_Mouse_First + 8 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_First: Button = unsafe { Button_Mouse_Last + 1 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Button_First: Button = unsafe { Button_Gamepad_First + 0 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_A: Button = unsafe { Button_Gamepad_First + 0 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_B: Button = unsafe { Button_Gamepad_First + 1 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_X: Button = unsafe { Button_Gamepad_First + 2 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Y: Button = unsafe { Button_Gamepad_First + 3 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Back: Button = unsafe { Button_Gamepad_First + 4 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Guide: Button = unsafe { Button_Gamepad_First + 5 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Start: Button = unsafe { Button_Gamepad_First + 6 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_LStick: Button = unsafe { Button_Gamepad_First + 7 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_RStick: Button = unsafe { Button_Gamepad_First + 8 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_LBumper: Button = unsafe { Button_Gamepad_First + 9 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_RBumper: Button = unsafe { Button_Gamepad_First + 10 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Up: Button = unsafe { Button_Gamepad_First + 11 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Down: Button = unsafe { Button_Gamepad_First + 12 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Left: Button = unsafe { Button_Gamepad_First + 13 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Right: Button = unsafe { Button_Gamepad_First + 14 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Button_Last: Button = unsafe { Button_Gamepad_First + 14 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Axis_First: Button = unsafe { Button_Gamepad_First + 15 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_LTrigger: Button = unsafe { Button_Gamepad_First + 15 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_RTrigger: Button = unsafe { Button_Gamepad_First + 16 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_LStickX: Button = unsafe { Button_Gamepad_First + 17 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_LStickY: Button = unsafe { Button_Gamepad_First + 18 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_RStickX: Button = unsafe { Button_Gamepad_First + 19 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_RStickY: Button = unsafe { Button_Gamepad_First + 20 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Axis_Last: Button = unsafe { Button_Gamepad_First + 20 as i32 };
#[no_mangle]
pub static mut Button_Gamepad_Last: Button = unsafe { Button_Gamepad_First + 20 as i32 };
#[no_mangle]
pub static mut Button_System_First: Button = unsafe { Button_Gamepad_Last + 1 as i32 };
#[no_mangle]
pub static mut Button_System_Exit: Button = unsafe { Button_System_First + 0 as i32 };
#[no_mangle]
pub static mut Button_System_Last: Button = unsafe { Button_System_First + 0 as i32 };
#[no_mangle]
pub static mut Button_Last: Button = unsafe { Button_System_Last };
#[no_mangle]
pub unsafe extern "C" fn Button_ToDeviceType(mut button: Button) -> DeviceType {
    if button == Button_Null {
        return DeviceType_Null;
    } else if button <= Button_Keyboard_Last {
        return DeviceType_Keyboard;
    } else if button <= Button_Mouse_Last {
        return DeviceType_Mouse;
    } else if button <= Button_Gamepad_Last {
        return DeviceType_Gamepad;
    } else if button <= Button_System_Last {
        return DeviceType_Null;
    } else {
        Fatal(
            b"Button_ToDeviceType: Unknown Button: %i\0" as *const u8 as *const libc::c_char,
            button,
        );
        return DeviceType_Null;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToString(mut button: Button) -> *const libc::c_char {
    match button {
        0 => return b"Button_Null\0" as *const u8 as *const libc::c_char,
        1 => return b"Button_Keyboard_A\0" as *const u8 as *const libc::c_char,
        2 => return b"Button_Keyboard_B\0" as *const u8 as *const libc::c_char,
        3 => return b"Button_Keyboard_C\0" as *const u8 as *const libc::c_char,
        4 => return b"Button_Keyboard_D\0" as *const u8 as *const libc::c_char,
        5 => return b"Button_Keyboard_E\0" as *const u8 as *const libc::c_char,
        6 => return b"Button_Keyboard_F\0" as *const u8 as *const libc::c_char,
        7 => return b"Button_Keyboard_G\0" as *const u8 as *const libc::c_char,
        8 => return b"Button_Keyboard_H\0" as *const u8 as *const libc::c_char,
        9 => return b"Button_Keyboard_I\0" as *const u8 as *const libc::c_char,
        10 => return b"Button_Keyboard_J\0" as *const u8 as *const libc::c_char,
        11 => return b"Button_Keyboard_K\0" as *const u8 as *const libc::c_char,
        12 => return b"Button_Keyboard_L\0" as *const u8 as *const libc::c_char,
        13 => return b"Button_Keyboard_M\0" as *const u8 as *const libc::c_char,
        14 => return b"Button_Keyboard_N\0" as *const u8 as *const libc::c_char,
        15 => return b"Button_Keyboard_O\0" as *const u8 as *const libc::c_char,
        16 => return b"Button_Keyboard_P\0" as *const u8 as *const libc::c_char,
        17 => return b"Button_Keyboard_Q\0" as *const u8 as *const libc::c_char,
        18 => return b"Button_Keyboard_R\0" as *const u8 as *const libc::c_char,
        19 => return b"Button_Keyboard_S\0" as *const u8 as *const libc::c_char,
        20 => return b"Button_Keyboard_T\0" as *const u8 as *const libc::c_char,
        21 => return b"Button_Keyboard_U\0" as *const u8 as *const libc::c_char,
        22 => return b"Button_Keyboard_V\0" as *const u8 as *const libc::c_char,
        23 => return b"Button_Keyboard_W\0" as *const u8 as *const libc::c_char,
        24 => return b"Button_Keyboard_X\0" as *const u8 as *const libc::c_char,
        25 => return b"Button_Keyboard_Y\0" as *const u8 as *const libc::c_char,
        26 => return b"Button_Keyboard_Z\0" as *const u8 as *const libc::c_char,
        27 => return b"Button_Keyboard_N0\0" as *const u8 as *const libc::c_char,
        28 => return b"Button_Keyboard_N1\0" as *const u8 as *const libc::c_char,
        29 => return b"Button_Keyboard_N2\0" as *const u8 as *const libc::c_char,
        30 => return b"Button_Keyboard_N3\0" as *const u8 as *const libc::c_char,
        31 => return b"Button_Keyboard_N4\0" as *const u8 as *const libc::c_char,
        32 => return b"Button_Keyboard_N5\0" as *const u8 as *const libc::c_char,
        33 => return b"Button_Keyboard_N6\0" as *const u8 as *const libc::c_char,
        34 => return b"Button_Keyboard_N7\0" as *const u8 as *const libc::c_char,
        35 => return b"Button_Keyboard_N8\0" as *const u8 as *const libc::c_char,
        36 => return b"Button_Keyboard_N9\0" as *const u8 as *const libc::c_char,
        37 => return b"Button_Keyboard_F1\0" as *const u8 as *const libc::c_char,
        38 => return b"Button_Keyboard_F2\0" as *const u8 as *const libc::c_char,
        39 => return b"Button_Keyboard_F3\0" as *const u8 as *const libc::c_char,
        40 => return b"Button_Keyboard_F4\0" as *const u8 as *const libc::c_char,
        41 => return b"Button_Keyboard_F5\0" as *const u8 as *const libc::c_char,
        42 => return b"Button_Keyboard_F6\0" as *const u8 as *const libc::c_char,
        43 => return b"Button_Keyboard_F7\0" as *const u8 as *const libc::c_char,
        44 => return b"Button_Keyboard_F8\0" as *const u8 as *const libc::c_char,
        45 => return b"Button_Keyboard_F9\0" as *const u8 as *const libc::c_char,
        46 => return b"Button_Keyboard_F10\0" as *const u8 as *const libc::c_char,
        47 => return b"Button_Keyboard_F11\0" as *const u8 as *const libc::c_char,
        48 => return b"Button_Keyboard_F12\0" as *const u8 as *const libc::c_char,
        49 => return b"Button_Keyboard_F13\0" as *const u8 as *const libc::c_char,
        50 => return b"Button_Keyboard_F14\0" as *const u8 as *const libc::c_char,
        51 => return b"Button_Keyboard_F15\0" as *const u8 as *const libc::c_char,
        52 => return b"Button_Keyboard_F16\0" as *const u8 as *const libc::c_char,
        53 => return b"Button_Keyboard_F17\0" as *const u8 as *const libc::c_char,
        54 => return b"Button_Keyboard_F18\0" as *const u8 as *const libc::c_char,
        55 => return b"Button_Keyboard_F19\0" as *const u8 as *const libc::c_char,
        56 => return b"Button_Keyboard_F20\0" as *const u8 as *const libc::c_char,
        57 => return b"Button_Keyboard_F21\0" as *const u8 as *const libc::c_char,
        58 => return b"Button_Keyboard_F22\0" as *const u8 as *const libc::c_char,
        59 => return b"Button_Keyboard_F23\0" as *const u8 as *const libc::c_char,
        60 => return b"Button_Keyboard_F24\0" as *const u8 as *const libc::c_char,
        61 => return b"Button_Keyboard_KP0\0" as *const u8 as *const libc::c_char,
        62 => return b"Button_Keyboard_KP1\0" as *const u8 as *const libc::c_char,
        63 => return b"Button_Keyboard_KP2\0" as *const u8 as *const libc::c_char,
        64 => return b"Button_Keyboard_KP3\0" as *const u8 as *const libc::c_char,
        65 => return b"Button_Keyboard_KP4\0" as *const u8 as *const libc::c_char,
        66 => return b"Button_Keyboard_KP5\0" as *const u8 as *const libc::c_char,
        67 => return b"Button_Keyboard_KP6\0" as *const u8 as *const libc::c_char,
        68 => return b"Button_Keyboard_KP7\0" as *const u8 as *const libc::c_char,
        69 => return b"Button_Keyboard_KP8\0" as *const u8 as *const libc::c_char,
        70 => return b"Button_Keyboard_KP9\0" as *const u8 as *const libc::c_char,
        71 => return b"Button_Keyboard_KPNumLock\0" as *const u8 as *const libc::c_char,
        72 => return b"Button_Keyboard_KPDivide\0" as *const u8 as *const libc::c_char,
        73 => return b"Button_Keyboard_KPMultiply\0" as *const u8 as *const libc::c_char,
        74 => return b"Button_Keyboard_KPSubtract\0" as *const u8 as *const libc::c_char,
        75 => return b"Button_Keyboard_KPAdd\0" as *const u8 as *const libc::c_char,
        76 => return b"Button_Keyboard_KPEnter\0" as *const u8 as *const libc::c_char,
        77 => return b"Button_Keyboard_KPDecimal\0" as *const u8 as *const libc::c_char,
        78 => return b"Button_Keyboard_Backspace\0" as *const u8 as *const libc::c_char,
        79 => return b"Button_Keyboard_Escape\0" as *const u8 as *const libc::c_char,
        80 => return b"Button_Keyboard_Return\0" as *const u8 as *const libc::c_char,
        81 => return b"Button_Keyboard_Space\0" as *const u8 as *const libc::c_char,
        82 => return b"Button_Keyboard_Tab\0" as *const u8 as *const libc::c_char,
        83 => return b"Button_Keyboard_Backtick\0" as *const u8 as *const libc::c_char,
        84 => return b"Button_Keyboard_CapsLock\0" as *const u8 as *const libc::c_char,
        85 => return b"Button_Keyboard_Minus\0" as *const u8 as *const libc::c_char,
        86 => return b"Button_Keyboard_Equals\0" as *const u8 as *const libc::c_char,
        87 => return b"Button_Keyboard_LBracket\0" as *const u8 as *const libc::c_char,
        88 => return b"Button_Keyboard_RBracket\0" as *const u8 as *const libc::c_char,
        89 => return b"Button_Keyboard_Backslash\0" as *const u8 as *const libc::c_char,
        90 => return b"Button_Keyboard_Semicolon\0" as *const u8 as *const libc::c_char,
        91 => return b"Button_Keyboard_Apostrophe\0" as *const u8 as *const libc::c_char,
        92 => return b"Button_Keyboard_Comma\0" as *const u8 as *const libc::c_char,
        93 => return b"Button_Keyboard_Period\0" as *const u8 as *const libc::c_char,
        94 => return b"Button_Keyboard_Slash\0" as *const u8 as *const libc::c_char,
        95 => return b"Button_Keyboard_PrintScreen\0" as *const u8 as *const libc::c_char,
        96 => return b"Button_Keyboard_ScrollLock\0" as *const u8 as *const libc::c_char,
        97 => return b"Button_Keyboard_Pause\0" as *const u8 as *const libc::c_char,
        98 => return b"Button_Keyboard_Insert\0" as *const u8 as *const libc::c_char,
        99 => return b"Button_Keyboard_Delete\0" as *const u8 as *const libc::c_char,
        100 => return b"Button_Keyboard_Home\0" as *const u8 as *const libc::c_char,
        101 => return b"Button_Keyboard_End\0" as *const u8 as *const libc::c_char,
        102 => return b"Button_Keyboard_PageUp\0" as *const u8 as *const libc::c_char,
        103 => return b"Button_Keyboard_PageDown\0" as *const u8 as *const libc::c_char,
        104 => return b"Button_Keyboard_Right\0" as *const u8 as *const libc::c_char,
        105 => return b"Button_Keyboard_Left\0" as *const u8 as *const libc::c_char,
        106 => return b"Button_Keyboard_Down\0" as *const u8 as *const libc::c_char,
        107 => return b"Button_Keyboard_Up\0" as *const u8 as *const libc::c_char,
        108 => return b"Button_Keyboard_LCtrl\0" as *const u8 as *const libc::c_char,
        109 => return b"Button_Keyboard_LShift\0" as *const u8 as *const libc::c_char,
        110 => return b"Button_Keyboard_LAlt\0" as *const u8 as *const libc::c_char,
        111 => return b"Button_Keyboard_LMeta\0" as *const u8 as *const libc::c_char,
        112 => return b"Button_Keyboard_RCtrl\0" as *const u8 as *const libc::c_char,
        113 => return b"Button_Keyboard_RShift\0" as *const u8 as *const libc::c_char,
        114 => return b"Button_Keyboard_RAlt\0" as *const u8 as *const libc::c_char,
        115 => return b"Button_Keyboard_RMeta\0" as *const u8 as *const libc::c_char,
        117 => return b"Button_Mouse_Left\0" as *const u8 as *const libc::c_char,
        118 => return b"Button_Mouse_Middle\0" as *const u8 as *const libc::c_char,
        119 => return b"Button_Mouse_Right\0" as *const u8 as *const libc::c_char,
        120 => return b"Button_Mouse_X1\0" as *const u8 as *const libc::c_char,
        121 => return b"Button_Mouse_X2\0" as *const u8 as *const libc::c_char,
        122 => return b"Button_Mouse_X\0" as *const u8 as *const libc::c_char,
        123 => return b"Button_Mouse_Y\0" as *const u8 as *const libc::c_char,
        124 => return b"Button_Mouse_ScrollX\0" as *const u8 as *const libc::c_char,
        125 => return b"Button_Mouse_ScrollY\0" as *const u8 as *const libc::c_char,
        126 => return b"Button_Gamepad_A\0" as *const u8 as *const libc::c_char,
        127 => return b"Button_Gamepad_B\0" as *const u8 as *const libc::c_char,
        128 => return b"Button_Gamepad_X\0" as *const u8 as *const libc::c_char,
        129 => return b"Button_Gamepad_Y\0" as *const u8 as *const libc::c_char,
        130 => return b"Button_Gamepad_Back\0" as *const u8 as *const libc::c_char,
        131 => return b"Button_Gamepad_Guide\0" as *const u8 as *const libc::c_char,
        132 => return b"Button_Gamepad_Start\0" as *const u8 as *const libc::c_char,
        133 => return b"Button_Gamepad_LStick\0" as *const u8 as *const libc::c_char,
        134 => return b"Button_Gamepad_RStick\0" as *const u8 as *const libc::c_char,
        135 => return b"Button_Gamepad_LBumper\0" as *const u8 as *const libc::c_char,
        136 => return b"Button_Gamepad_RBumper\0" as *const u8 as *const libc::c_char,
        137 => return b"Button_Gamepad_Up\0" as *const u8 as *const libc::c_char,
        138 => return b"Button_Gamepad_Down\0" as *const u8 as *const libc::c_char,
        139 => return b"Button_Gamepad_Left\0" as *const u8 as *const libc::c_char,
        140 => return b"Button_Gamepad_Right\0" as *const u8 as *const libc::c_char,
        141 => return b"Button_Gamepad_LTrigger\0" as *const u8 as *const libc::c_char,
        142 => return b"Button_Gamepad_RTrigger\0" as *const u8 as *const libc::c_char,
        143 => return b"Button_Gamepad_LStickX\0" as *const u8 as *const libc::c_char,
        144 => return b"Button_Gamepad_LStickY\0" as *const u8 as *const libc::c_char,
        145 => return b"Button_Gamepad_RStickX\0" as *const u8 as *const libc::c_char,
        146 => return b"Button_Gamepad_RStickY\0" as *const u8 as *const libc::c_char,
        147 => return b"Button_System_Exit\0" as *const u8 as *const libc::c_char,
        _ => {
            static mut buffer: [libc::c_char; 512] = [0; 512];
            libc::snprintf(
                buffer.as_mut_ptr(),
                (::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(::core::mem::size_of::<libc::c_char>() as usize)
                    as i32 as usize,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                button,
            );
            return buffer.as_mut_ptr() as *const libc::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_IsAutoRelease(mut button: Button) -> bool {
    match button {
        124 | 125 | 147 => return 1 as i32 != 0,
        _ => return 0 as i32 != 0,
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLScancode(mut scancode: SDL_Scancode) -> Button {
    match scancode as u32 {
        0 => return Button_Null,
        4 => return Button_Keyboard_A,
        5 => return Button_Keyboard_B,
        6 => return Button_Keyboard_C,
        7 => return Button_Keyboard_D,
        8 => return Button_Keyboard_E,
        9 => return Button_Keyboard_F,
        10 => return Button_Keyboard_G,
        11 => return Button_Keyboard_H,
        12 => return Button_Keyboard_I,
        13 => return Button_Keyboard_J,
        14 => return Button_Keyboard_K,
        15 => return Button_Keyboard_L,
        16 => return Button_Keyboard_M,
        17 => return Button_Keyboard_N,
        18 => return Button_Keyboard_O,
        19 => return Button_Keyboard_P,
        20 => return Button_Keyboard_Q,
        21 => return Button_Keyboard_R,
        22 => return Button_Keyboard_S,
        23 => return Button_Keyboard_T,
        24 => return Button_Keyboard_U,
        25 => return Button_Keyboard_V,
        26 => return Button_Keyboard_W,
        27 => return Button_Keyboard_X,
        28 => return Button_Keyboard_Y,
        29 => return Button_Keyboard_Z,
        39 => return Button_Keyboard_N0,
        30 => return Button_Keyboard_N1,
        31 => return Button_Keyboard_N2,
        32 => return Button_Keyboard_N3,
        33 => return Button_Keyboard_N4,
        34 => return Button_Keyboard_N5,
        35 => return Button_Keyboard_N6,
        36 => return Button_Keyboard_N7,
        37 => return Button_Keyboard_N8,
        38 => return Button_Keyboard_N9,
        58 => return Button_Keyboard_F1,
        59 => return Button_Keyboard_F2,
        60 => return Button_Keyboard_F3,
        61 => return Button_Keyboard_F4,
        62 => return Button_Keyboard_F5,
        63 => return Button_Keyboard_F6,
        64 => return Button_Keyboard_F7,
        65 => return Button_Keyboard_F8,
        66 => return Button_Keyboard_F9,
        67 => return Button_Keyboard_F10,
        68 => return Button_Keyboard_F11,
        69 => return Button_Keyboard_F12,
        104 => return Button_Keyboard_F13,
        105 => return Button_Keyboard_F14,
        106 => return Button_Keyboard_F15,
        107 => return Button_Keyboard_F16,
        108 => return Button_Keyboard_F17,
        109 => return Button_Keyboard_F18,
        110 => return Button_Keyboard_F19,
        111 => return Button_Keyboard_F20,
        112 => return Button_Keyboard_F21,
        113 => return Button_Keyboard_F22,
        114 => return Button_Keyboard_F23,
        115 => return Button_Keyboard_F24,
        98 => return Button_Keyboard_KP0,
        89 => return Button_Keyboard_KP1,
        90 => return Button_Keyboard_KP2,
        91 => return Button_Keyboard_KP3,
        92 => return Button_Keyboard_KP4,
        93 => return Button_Keyboard_KP5,
        94 => return Button_Keyboard_KP6,
        95 => return Button_Keyboard_KP7,
        96 => return Button_Keyboard_KP8,
        97 => return Button_Keyboard_KP9,
        83 => return Button_Keyboard_KPNumLock,
        84 => return Button_Keyboard_KPDivide,
        85 => return Button_Keyboard_KPMultiply,
        86 => return Button_Keyboard_KPSubtract,
        87 => return Button_Keyboard_KPAdd,
        88 => return Button_Keyboard_KPEnter,
        220 => return Button_Keyboard_KPDecimal,
        42 => return Button_Keyboard_Backspace,
        41 => return Button_Keyboard_Escape,
        40 => return Button_Keyboard_Return,
        44 => return Button_Keyboard_Space,
        43 => return Button_Keyboard_Tab,
        53 => return Button_Keyboard_Backtick,
        57 => return Button_Keyboard_CapsLock,
        45 => return Button_Keyboard_Minus,
        46 => return Button_Keyboard_Equals,
        47 => return Button_Keyboard_LBracket,
        48 => return Button_Keyboard_RBracket,
        49 => return Button_Keyboard_Backslash,
        51 => return Button_Keyboard_Semicolon,
        52 => return Button_Keyboard_Apostrophe,
        54 => return Button_Keyboard_Comma,
        55 => return Button_Keyboard_Period,
        56 => return Button_Keyboard_Slash,
        70 => return Button_Keyboard_PrintScreen,
        71 => return Button_Keyboard_ScrollLock,
        72 => return Button_Keyboard_Pause,
        73 => return Button_Keyboard_Insert,
        76 => return Button_Keyboard_Delete,
        74 => return Button_Keyboard_Home,
        77 => return Button_Keyboard_End,
        75 => return Button_Keyboard_PageUp,
        78 => return Button_Keyboard_PageDown,
        79 => return Button_Keyboard_Right,
        80 => return Button_Keyboard_Left,
        81 => return Button_Keyboard_Down,
        82 => return Button_Keyboard_Up,
        224 => return Button_Keyboard_LCtrl,
        225 => return Button_Keyboard_LShift,
        226 => return Button_Keyboard_LAlt,
        227 => return Button_Keyboard_LMeta,
        228 => return Button_Keyboard_RCtrl,
        229 => return Button_Keyboard_RShift,
        230 => return Button_Keyboard_RAlt,
        231 => return Button_Keyboard_RMeta,
        _ => return Button_Null,
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLScancode(mut button: Button) -> SDL_Scancode {
    match button {
        0 => {}
        1 => return SDL_Scancode::SDL_SCANCODE_A,
        2 => return SDL_Scancode::SDL_SCANCODE_B,
        3 => return SDL_Scancode::SDL_SCANCODE_C,
        4 => return SDL_Scancode::SDL_SCANCODE_D,
        5 => return SDL_Scancode::SDL_SCANCODE_E,
        6 => return SDL_Scancode::SDL_SCANCODE_F,
        7 => return SDL_Scancode::SDL_SCANCODE_G,
        8 => return SDL_Scancode::SDL_SCANCODE_H,
        9 => return SDL_Scancode::SDL_SCANCODE_I,
        10 => return SDL_Scancode::SDL_SCANCODE_J,
        11 => return SDL_Scancode::SDL_SCANCODE_K,
        12 => return SDL_Scancode::SDL_SCANCODE_L,
        13 => return SDL_Scancode::SDL_SCANCODE_M,
        14 => return SDL_Scancode::SDL_SCANCODE_N,
        15 => return SDL_Scancode::SDL_SCANCODE_O,
        16 => return SDL_Scancode::SDL_SCANCODE_P,
        17 => return SDL_Scancode::SDL_SCANCODE_Q,
        18 => return SDL_Scancode::SDL_SCANCODE_R,
        19 => return SDL_Scancode::SDL_SCANCODE_S,
        20 => return SDL_Scancode::SDL_SCANCODE_T,
        21 => return SDL_Scancode::SDL_SCANCODE_U,
        22 => return SDL_Scancode::SDL_SCANCODE_V,
        23 => return SDL_Scancode::SDL_SCANCODE_W,
        24 => return SDL_Scancode::SDL_SCANCODE_X,
        25 => return SDL_Scancode::SDL_SCANCODE_Y,
        26 => return SDL_Scancode::SDL_SCANCODE_Z,
        27 => return SDL_Scancode::SDL_SCANCODE_0,
        28 => return SDL_Scancode::SDL_SCANCODE_1,
        29 => return SDL_Scancode::SDL_SCANCODE_2,
        30 => return SDL_Scancode::SDL_SCANCODE_3,
        31 => return SDL_Scancode::SDL_SCANCODE_4,
        32 => return SDL_Scancode::SDL_SCANCODE_5,
        33 => return SDL_Scancode::SDL_SCANCODE_6,
        34 => return SDL_Scancode::SDL_SCANCODE_7,
        35 => return SDL_Scancode::SDL_SCANCODE_8,
        36 => return SDL_Scancode::SDL_SCANCODE_9,
        37 => return SDL_Scancode::SDL_SCANCODE_F1,
        38 => return SDL_Scancode::SDL_SCANCODE_F2,
        39 => return SDL_Scancode::SDL_SCANCODE_F3,
        40 => return SDL_Scancode::SDL_SCANCODE_F4,
        41 => return SDL_Scancode::SDL_SCANCODE_F5,
        42 => return SDL_Scancode::SDL_SCANCODE_F6,
        43 => return SDL_Scancode::SDL_SCANCODE_F7,
        44 => return SDL_Scancode::SDL_SCANCODE_F8,
        45 => return SDL_Scancode::SDL_SCANCODE_F9,
        46 => return SDL_Scancode::SDL_SCANCODE_F10,
        47 => return SDL_Scancode::SDL_SCANCODE_F11,
        48 => return SDL_Scancode::SDL_SCANCODE_F12,
        49 => return SDL_Scancode::SDL_SCANCODE_F13,
        50 => return SDL_Scancode::SDL_SCANCODE_F14,
        51 => return SDL_Scancode::SDL_SCANCODE_F15,
        52 => return SDL_Scancode::SDL_SCANCODE_F16,
        53 => return SDL_Scancode::SDL_SCANCODE_F17,
        54 => return SDL_Scancode::SDL_SCANCODE_F18,
        55 => return SDL_Scancode::SDL_SCANCODE_F19,
        56 => return SDL_Scancode::SDL_SCANCODE_F20,
        57 => return SDL_Scancode::SDL_SCANCODE_F21,
        58 => return SDL_Scancode::SDL_SCANCODE_F22,
        59 => return SDL_Scancode::SDL_SCANCODE_F23,
        60 => return SDL_Scancode::SDL_SCANCODE_F24,
        61 => return SDL_Scancode::SDL_SCANCODE_KP_0,
        62 => return SDL_Scancode::SDL_SCANCODE_KP_1,
        63 => return SDL_Scancode::SDL_SCANCODE_KP_2,
        64 => return SDL_Scancode::SDL_SCANCODE_KP_3,
        65 => return SDL_Scancode::SDL_SCANCODE_KP_4,
        66 => return SDL_Scancode::SDL_SCANCODE_KP_5,
        67 => return SDL_Scancode::SDL_SCANCODE_KP_6,
        68 => return SDL_Scancode::SDL_SCANCODE_KP_7,
        69 => return SDL_Scancode::SDL_SCANCODE_KP_8,
        70 => return SDL_Scancode::SDL_SCANCODE_KP_9,
        71 => return SDL_Scancode::SDL_SCANCODE_NUMLOCKCLEAR,
        72 => return SDL_Scancode::SDL_SCANCODE_KP_DIVIDE,
        73 => return SDL_Scancode::SDL_SCANCODE_KP_MULTIPLY,
        74 => return SDL_Scancode::SDL_SCANCODE_KP_MINUS,
        75 => return SDL_Scancode::SDL_SCANCODE_KP_PLUS,
        76 => return SDL_Scancode::SDL_SCANCODE_KP_ENTER,
        77 => return SDL_Scancode::SDL_SCANCODE_KP_DECIMAL,
        78 => return SDL_Scancode::SDL_SCANCODE_BACKSPACE,
        79 => return SDL_Scancode::SDL_SCANCODE_ESCAPE,
        80 => return SDL_Scancode::SDL_SCANCODE_RETURN,
        81 => return SDL_Scancode::SDL_SCANCODE_SPACE,
        82 => return SDL_Scancode::SDL_SCANCODE_TAB,
        83 => return SDL_Scancode::SDL_SCANCODE_GRAVE,
        84 => return SDL_Scancode::SDL_SCANCODE_CAPSLOCK,
        85 => return SDL_Scancode::SDL_SCANCODE_MINUS,
        86 => return SDL_Scancode::SDL_SCANCODE_EQUALS,
        87 => return SDL_Scancode::SDL_SCANCODE_LEFTBRACKET,
        88 => return SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET,
        89 => return SDL_Scancode::SDL_SCANCODE_BACKSLASH,
        90 => return SDL_Scancode::SDL_SCANCODE_SEMICOLON,
        91 => return SDL_Scancode::SDL_SCANCODE_APOSTROPHE,
        92 => return SDL_Scancode::SDL_SCANCODE_COMMA,
        93 => return SDL_Scancode::SDL_SCANCODE_PERIOD,
        94 => return SDL_Scancode::SDL_SCANCODE_SLASH,
        95 => return SDL_Scancode::SDL_SCANCODE_PRINTSCREEN,
        96 => return SDL_Scancode::SDL_SCANCODE_SCROLLLOCK,
        97 => return SDL_Scancode::SDL_SCANCODE_PAUSE,
        98 => return SDL_Scancode::SDL_SCANCODE_INSERT,
        99 => return SDL_Scancode::SDL_SCANCODE_DELETE,
        100 => return SDL_Scancode::SDL_SCANCODE_HOME,
        101 => return SDL_Scancode::SDL_SCANCODE_END,
        102 => return SDL_Scancode::SDL_SCANCODE_PAGEUP,
        103 => return SDL_Scancode::SDL_SCANCODE_PAGEDOWN,
        104 => return SDL_Scancode::SDL_SCANCODE_RIGHT,
        105 => return SDL_Scancode::SDL_SCANCODE_LEFT,
        106 => return SDL_Scancode::SDL_SCANCODE_DOWN,
        107 => return SDL_Scancode::SDL_SCANCODE_UP,
        108 => return SDL_Scancode::SDL_SCANCODE_LCTRL,
        109 => return SDL_Scancode::SDL_SCANCODE_LSHIFT,
        110 => return SDL_Scancode::SDL_SCANCODE_LALT,
        111 => return SDL_Scancode::SDL_SCANCODE_LGUI,
        112 => return SDL_Scancode::SDL_SCANCODE_RCTRL,
        113 => return SDL_Scancode::SDL_SCANCODE_RSHIFT,
        114 => return SDL_Scancode::SDL_SCANCODE_RALT,
        115 => return SDL_Scancode::SDL_SCANCODE_RGUI,
        _ => {
            Fatal(
                b"Button_ToSDLScancode: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_Scancode::SDL_SCANCODE_UNKNOWN;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLMouseButton(mut mouseButton: u8) -> Button {
    match mouseButton as i32 {
        1 => {}
        2 => return Button_Mouse_Middle,
        3 => return Button_Mouse_Right,
        4 => return Button_Mouse_X1,
        5 => return Button_Mouse_X2,
        _ => {
            Fatal(
                b"Button_FromSDLMouseButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                mouseButton as i32,
            );
        }
    }
    return Button_Mouse_Left;
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLMouseButton(mut button: Button) -> u8 {
    match button {
        117 => {}
        118 => return 2 as i32 as u8,
        119 => return 3 as i32 as u8,
        120 => return 4 as i32 as u8,
        121 => return 5 as i32 as u8,
        _ => {
            Fatal(
                b"Button_ToSDLMouseButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return 1 as i32 as u8;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerAxis(
    mut controllerAxis: SDL_GameControllerAxis,
) -> Button {
    match controllerAxis as i32 {
        0 => {}
        1 => return Button_Gamepad_LStickY,
        2 => return Button_Gamepad_RStickX,
        3 => return Button_Gamepad_RStickY,
        4 => return Button_Gamepad_LTrigger,
        5 => return Button_Gamepad_RTrigger,
        _ => {
            Fatal(
                b"Button_FromSDLControllerAxis: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                controllerAxis as i32,
            );
        }
    }
    return Button_Gamepad_LStickX;
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLControllerAxis(mut button: Button) -> SDL_GameControllerAxis {
    match button {
        143 => {}
        144 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY,
        145 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX,
        146 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY,
        141 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT,
        142 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT,
        _ => {
            Fatal(
                b"Button_ToSDLControllerAxis: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerButton(
    mut controllerButton: SDL_GameControllerButton,
) -> Button {
    match controllerButton as i32 {
        0 => {}
        1 => return Button_Gamepad_B,
        2 => return Button_Gamepad_X,
        3 => return Button_Gamepad_Y,
        4 => return Button_Gamepad_Back,
        5 => return Button_Gamepad_Guide,
        6 => return Button_Gamepad_Start,
        7 => return Button_Gamepad_LStick,
        8 => return Button_Gamepad_RStick,
        9 => return Button_Gamepad_LBumper,
        10 => return Button_Gamepad_RBumper,
        11 => return Button_Gamepad_Up,
        12 => return Button_Gamepad_Down,
        13 => return Button_Gamepad_Left,
        14 => return Button_Gamepad_Right,
        _ => {
            Fatal(
                b"Button_FromSDLControllerButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                controllerButton as i32,
            );
        }
    }
    return Button_Gamepad_A;
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLControllerButton(
    mut button: Button,
) -> SDL_GameControllerButton {
    match button {
        126 => {}
        127 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B,
        128 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X,
        129 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y,
        130 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK,
        131 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_GUIDE,
        132 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START,
        133 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK,
        134 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK,
        135 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
        136 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
        137 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP,
        138 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN,
        139 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT,
        140 => return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
        _ => {
            Fatal(
                b"Button_ToSDLControllerButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A;
}
