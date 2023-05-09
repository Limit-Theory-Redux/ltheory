use crate::internal::ffi;

use crate::Common::*;
use crate::DeviceType::*;
use sdl2_sys::*;

pub type Button = i32;

#[no_mangle]
pub static Button_Null: Button = 0;

#[no_mangle]
pub static Button_First: Button = 1;

#[no_mangle]
pub static Button_Keyboard_First: Button = Button_First;

#[no_mangle]
pub static Button_Keyboard_A: Button = Button_Keyboard_First + 0;

#[no_mangle]
pub static Button_Keyboard_B: Button = Button_Keyboard_First + 1;

#[no_mangle]
pub static Button_Keyboard_C: Button = Button_Keyboard_First + 2;

#[no_mangle]
pub static Button_Keyboard_D: Button = Button_Keyboard_First + 3;

#[no_mangle]
pub static Button_Keyboard_E: Button = Button_Keyboard_First + 4;

#[no_mangle]
pub static Button_Keyboard_F: Button = Button_Keyboard_First + 5;

#[no_mangle]
pub static Button_Keyboard_G: Button = Button_Keyboard_First + 6;

#[no_mangle]
pub static Button_Keyboard_H: Button = Button_Keyboard_First + 7;

#[no_mangle]
pub static Button_Keyboard_I: Button = Button_Keyboard_First + 8;

#[no_mangle]
pub static Button_Keyboard_J: Button = Button_Keyboard_First + 9;

#[no_mangle]
pub static Button_Keyboard_K: Button = Button_Keyboard_First + 10;

#[no_mangle]
pub static Button_Keyboard_L: Button = Button_Keyboard_First + 11;

#[no_mangle]
pub static Button_Keyboard_M: Button = Button_Keyboard_First + 12;

#[no_mangle]
pub static Button_Keyboard_N: Button = Button_Keyboard_First + 13;

#[no_mangle]
pub static Button_Keyboard_O: Button = Button_Keyboard_First + 14;

#[no_mangle]
pub static Button_Keyboard_P: Button = Button_Keyboard_First + 15;

#[no_mangle]
pub static Button_Keyboard_Q: Button = Button_Keyboard_First + 16;

#[no_mangle]
pub static Button_Keyboard_R: Button = Button_Keyboard_First + 17;

#[no_mangle]
pub static Button_Keyboard_S: Button = Button_Keyboard_First + 18;

#[no_mangle]
pub static Button_Keyboard_T: Button = Button_Keyboard_First + 19;

#[no_mangle]
pub static Button_Keyboard_U: Button = Button_Keyboard_First + 20;

#[no_mangle]
pub static Button_Keyboard_V: Button = Button_Keyboard_First + 21;

#[no_mangle]
pub static Button_Keyboard_W: Button = Button_Keyboard_First + 22;

#[no_mangle]
pub static Button_Keyboard_X: Button = Button_Keyboard_First + 23;

#[no_mangle]
pub static Button_Keyboard_Y: Button = Button_Keyboard_First + 24;

#[no_mangle]
pub static Button_Keyboard_Z: Button = Button_Keyboard_First + 25;

#[no_mangle]
pub static Button_Keyboard_N0: Button = Button_Keyboard_First + 26;

#[no_mangle]
pub static Button_Keyboard_N1: Button = Button_Keyboard_First + 27;

#[no_mangle]
pub static Button_Keyboard_N2: Button = Button_Keyboard_First + 28;

#[no_mangle]
pub static Button_Keyboard_N3: Button = Button_Keyboard_First + 29;

#[no_mangle]
pub static Button_Keyboard_N4: Button = Button_Keyboard_First + 30;

#[no_mangle]
pub static Button_Keyboard_N5: Button = Button_Keyboard_First + 31;

#[no_mangle]
pub static Button_Keyboard_N6: Button = Button_Keyboard_First + 32;

#[no_mangle]
pub static Button_Keyboard_N7: Button = Button_Keyboard_First + 33;

#[no_mangle]
pub static Button_Keyboard_N8: Button = Button_Keyboard_First + 34;

#[no_mangle]
pub static Button_Keyboard_N9: Button = Button_Keyboard_First + 35;

#[no_mangle]
pub static Button_Keyboard_F1: Button = Button_Keyboard_First + 36;

#[no_mangle]
pub static Button_Keyboard_F2: Button = Button_Keyboard_First + 37;

#[no_mangle]
pub static Button_Keyboard_F3: Button = Button_Keyboard_First + 38;

#[no_mangle]
pub static Button_Keyboard_F4: Button = Button_Keyboard_First + 39;

#[no_mangle]
pub static Button_Keyboard_F5: Button = Button_Keyboard_First + 40;

#[no_mangle]
pub static Button_Keyboard_F6: Button = Button_Keyboard_First + 41;

#[no_mangle]
pub static Button_Keyboard_F7: Button = Button_Keyboard_First + 42;

#[no_mangle]
pub static Button_Keyboard_F8: Button = Button_Keyboard_First + 43;

#[no_mangle]
pub static Button_Keyboard_F9: Button = Button_Keyboard_First + 44;

#[no_mangle]
pub static Button_Keyboard_F10: Button = Button_Keyboard_First + 45;

#[no_mangle]
pub static Button_Keyboard_F11: Button = Button_Keyboard_First + 46;

#[no_mangle]
pub static Button_Keyboard_F12: Button = Button_Keyboard_First + 47;

#[no_mangle]
pub static Button_Keyboard_F13: Button = Button_Keyboard_First + 48;

#[no_mangle]
pub static Button_Keyboard_F14: Button = Button_Keyboard_First + 49;

#[no_mangle]
pub static Button_Keyboard_F15: Button = Button_Keyboard_First + 50;

#[no_mangle]
pub static Button_Keyboard_F16: Button = Button_Keyboard_First + 51;

#[no_mangle]
pub static Button_Keyboard_F17: Button = Button_Keyboard_First + 52;

#[no_mangle]
pub static Button_Keyboard_F18: Button = Button_Keyboard_First + 53;

#[no_mangle]
pub static Button_Keyboard_F19: Button = Button_Keyboard_First + 54;

#[no_mangle]
pub static Button_Keyboard_F20: Button = Button_Keyboard_First + 55;

#[no_mangle]
pub static Button_Keyboard_F21: Button = Button_Keyboard_First + 56;

#[no_mangle]
pub static Button_Keyboard_F22: Button = Button_Keyboard_First + 57;

#[no_mangle]
pub static Button_Keyboard_F23: Button = Button_Keyboard_First + 58;

#[no_mangle]
pub static Button_Keyboard_F24: Button = Button_Keyboard_First + 59;

#[no_mangle]
pub static Button_Keyboard_KP0: Button = Button_Keyboard_First + 60;

#[no_mangle]
pub static Button_Keyboard_KP1: Button = Button_Keyboard_First + 61;

#[no_mangle]
pub static Button_Keyboard_KP2: Button = Button_Keyboard_First + 62;

#[no_mangle]
pub static Button_Keyboard_KP3: Button = Button_Keyboard_First + 63;

#[no_mangle]
pub static Button_Keyboard_KP4: Button = Button_Keyboard_First + 64;

#[no_mangle]
pub static Button_Keyboard_KP5: Button = Button_Keyboard_First + 65;

#[no_mangle]
pub static Button_Keyboard_KP6: Button = Button_Keyboard_First + 66;

#[no_mangle]
pub static Button_Keyboard_KP7: Button = Button_Keyboard_First + 67;

#[no_mangle]
pub static Button_Keyboard_KP8: Button = Button_Keyboard_First + 68;

#[no_mangle]
pub static Button_Keyboard_KP9: Button = Button_Keyboard_First + 69;

#[no_mangle]
pub static Button_Keyboard_KPNumLock: Button = Button_Keyboard_First + 70;

#[no_mangle]
pub static Button_Keyboard_KPDivide: Button = Button_Keyboard_First + 71;

#[no_mangle]
pub static Button_Keyboard_KPMultiply: Button = Button_Keyboard_First + 72;

#[no_mangle]
pub static Button_Keyboard_KPSubtract: Button = Button_Keyboard_First + 73;

#[no_mangle]
pub static Button_Keyboard_KPAdd: Button = Button_Keyboard_First + 74;

#[no_mangle]
pub static Button_Keyboard_KPEnter: Button = Button_Keyboard_First + 75;

#[no_mangle]
pub static Button_Keyboard_KPDecimal: Button = Button_Keyboard_First + 76;

#[no_mangle]
pub static Button_Keyboard_Backspace: Button = Button_Keyboard_First + 77;

#[no_mangle]
pub static Button_Keyboard_Escape: Button = Button_Keyboard_First + 78;

#[no_mangle]
pub static Button_Keyboard_Return: Button = Button_Keyboard_First + 79;

#[no_mangle]
pub static Button_Keyboard_Space: Button = Button_Keyboard_First + 80;

#[no_mangle]
pub static Button_Keyboard_Tab: Button = Button_Keyboard_First + 81;

#[no_mangle]
pub static Button_Keyboard_Backtick: Button = Button_Keyboard_First + 82;

#[no_mangle]
pub static Button_Keyboard_CapsLock: Button = Button_Keyboard_First + 83;

#[no_mangle]
pub static Button_Keyboard_Minus: Button = Button_Keyboard_First + 84;

#[no_mangle]
pub static Button_Keyboard_Equals: Button = Button_Keyboard_First + 85;

#[no_mangle]
pub static Button_Keyboard_LBracket: Button = Button_Keyboard_First + 86;

#[no_mangle]
pub static Button_Keyboard_RBracket: Button = Button_Keyboard_First + 87;

#[no_mangle]
pub static Button_Keyboard_Backslash: Button = Button_Keyboard_First + 88;

#[no_mangle]
pub static Button_Keyboard_Semicolon: Button = Button_Keyboard_First + 89;

#[no_mangle]
pub static Button_Keyboard_Apostrophe: Button = Button_Keyboard_First + 90;

#[no_mangle]
pub static Button_Keyboard_Comma: Button = Button_Keyboard_First + 91;

#[no_mangle]
pub static Button_Keyboard_Period: Button = Button_Keyboard_First + 92;

#[no_mangle]
pub static Button_Keyboard_Slash: Button = Button_Keyboard_First + 93;

#[no_mangle]
pub static Button_Keyboard_PrintScreen: Button = Button_Keyboard_First + 94;

#[no_mangle]
pub static Button_Keyboard_ScrollLock: Button = Button_Keyboard_First + 95;

#[no_mangle]
pub static Button_Keyboard_Pause: Button = Button_Keyboard_First + 96;

#[no_mangle]
pub static Button_Keyboard_Insert: Button = Button_Keyboard_First + 97;

#[no_mangle]
pub static Button_Keyboard_Delete: Button = Button_Keyboard_First + 98;

#[no_mangle]
pub static Button_Keyboard_Home: Button = Button_Keyboard_First + 99;

#[no_mangle]
pub static Button_Keyboard_End: Button = Button_Keyboard_First + 100;

#[no_mangle]
pub static Button_Keyboard_PageUp: Button = Button_Keyboard_First + 101;

#[no_mangle]
pub static Button_Keyboard_PageDown: Button = Button_Keyboard_First + 102;

#[no_mangle]
pub static Button_Keyboard_Right: Button = Button_Keyboard_First + 103;

#[no_mangle]
pub static Button_Keyboard_Left: Button = Button_Keyboard_First + 104;

#[no_mangle]
pub static Button_Keyboard_Down: Button = Button_Keyboard_First + 105;

#[no_mangle]
pub static Button_Keyboard_Up: Button = Button_Keyboard_First + 106;

#[no_mangle]
pub static Button_Keyboard_LCtrl: Button = Button_Keyboard_First + 107;

#[no_mangle]
pub static Button_Keyboard_LShift: Button = Button_Keyboard_First + 108;

#[no_mangle]
pub static Button_Keyboard_LAlt: Button = Button_Keyboard_First + 109;

#[no_mangle]
pub static Button_Keyboard_LMeta: Button = Button_Keyboard_First + 110;

#[no_mangle]
pub static Button_Keyboard_RCtrl: Button = Button_Keyboard_First + 111;

#[no_mangle]
pub static Button_Keyboard_RShift: Button = Button_Keyboard_First + 112;

#[no_mangle]
pub static Button_Keyboard_RAlt: Button = Button_Keyboard_First + 113;

#[no_mangle]
pub static Button_Keyboard_RMeta: Button = Button_Keyboard_First + 114;

#[no_mangle]
pub static Button_Keyboard_Last: Button = Button_Keyboard_First + 115;

#[no_mangle]
pub static Button_Mouse_First: Button = Button_Keyboard_Last + 1;

#[no_mangle]
pub static Button_Mouse_Left: Button = Button_Mouse_First + 0;

#[no_mangle]
pub static Button_Mouse_Middle: Button = Button_Mouse_First + 1;

#[no_mangle]
pub static Button_Mouse_Right: Button = Button_Mouse_First + 2;

#[no_mangle]
pub static Button_Mouse_X1: Button = Button_Mouse_First + 3;

#[no_mangle]
pub static Button_Mouse_X2: Button = Button_Mouse_First + 4;

#[no_mangle]
pub static Button_Mouse_X: Button = Button_Mouse_First + 5;

#[no_mangle]
pub static Button_Mouse_Y: Button = Button_Mouse_First + 6;

#[no_mangle]
pub static Button_Mouse_ScrollX: Button = Button_Mouse_First + 7;

#[no_mangle]
pub static Button_Mouse_ScrollY: Button = Button_Mouse_First + 8;

#[no_mangle]
pub static Button_Mouse_Last: Button = Button_Mouse_First + 8;

#[no_mangle]
pub static Button_Gamepad_First: Button = Button_Mouse_Last + 1;

#[no_mangle]
pub static Button_Gamepad_Button_First: Button = Button_Gamepad_First + 0;

#[no_mangle]
pub static Button_Gamepad_A: Button = Button_Gamepad_First + 0;

#[no_mangle]
pub static Button_Gamepad_B: Button = Button_Gamepad_First + 1;

#[no_mangle]
pub static Button_Gamepad_X: Button = Button_Gamepad_First + 2;

#[no_mangle]
pub static Button_Gamepad_Y: Button = Button_Gamepad_First + 3;

#[no_mangle]
pub static Button_Gamepad_Back: Button = Button_Gamepad_First + 4;

#[no_mangle]
pub static Button_Gamepad_Guide: Button = Button_Gamepad_First + 5;

#[no_mangle]
pub static Button_Gamepad_Start: Button = Button_Gamepad_First + 6;

#[no_mangle]
pub static Button_Gamepad_LStick: Button = Button_Gamepad_First + 7;

#[no_mangle]
pub static Button_Gamepad_RStick: Button = Button_Gamepad_First + 8;

#[no_mangle]
pub static Button_Gamepad_LBumper: Button = Button_Gamepad_First + 9;

#[no_mangle]
pub static Button_Gamepad_RBumper: Button = Button_Gamepad_First + 10;

#[no_mangle]
pub static Button_Gamepad_Up: Button = Button_Gamepad_First + 11;

#[no_mangle]
pub static Button_Gamepad_Down: Button = Button_Gamepad_First + 12;

#[no_mangle]
pub static Button_Gamepad_Left: Button = Button_Gamepad_First + 13;

#[no_mangle]
pub static Button_Gamepad_Right: Button = Button_Gamepad_First + 14;

#[no_mangle]
pub static Button_Gamepad_Button_Last: Button = Button_Gamepad_First + 14;

#[no_mangle]
pub static Button_Gamepad_Axis_First: Button = Button_Gamepad_First + 15;

#[no_mangle]
pub static Button_Gamepad_LTrigger: Button = Button_Gamepad_First + 15;

#[no_mangle]
pub static Button_Gamepad_RTrigger: Button = Button_Gamepad_First + 16;

#[no_mangle]
pub static Button_Gamepad_LStickX: Button = Button_Gamepad_First + 17;

#[no_mangle]
pub static Button_Gamepad_LStickY: Button = Button_Gamepad_First + 18;

#[no_mangle]
pub static Button_Gamepad_RStickX: Button = Button_Gamepad_First + 19;

#[no_mangle]
pub static Button_Gamepad_RStickY: Button = Button_Gamepad_First + 20;

#[no_mangle]
pub static Button_Gamepad_Axis_Last: Button = Button_Gamepad_First + 20;

#[no_mangle]
pub static Button_Gamepad_Last: Button = Button_Gamepad_First + 20;

#[no_mangle]
pub static Button_System_First: Button = Button_Gamepad_Last + 1;

#[no_mangle]
pub static Button_System_Exit: Button = Button_System_First + 0;

#[no_mangle]
pub static Button_System_Win_Enter: Button = Button_System_First + 1;

#[no_mangle]
pub static Button_System_Win_Leave: Button = Button_System_First + 2;

#[no_mangle]
pub static Button_System_Last: Button = Button_System_First + 2;

#[no_mangle]
pub static Button_Last: Button = Button_System_Last;

#[no_mangle]
pub extern "C" fn Button_ToDeviceType(button: Button) -> DeviceType {
    if button == Button_Null {
        DeviceType_Null
    } else if button <= Button_Keyboard_Last {
        return DeviceType_Keyboard;
    } else if button <= Button_Mouse_Last {
        return DeviceType_Mouse;
    } else if button <= Button_Gamepad_Last {
        return DeviceType_Gamepad;
    } else if button <= Button_System_Last {
        return DeviceType_Null;
    } else {
        CFatal!("Button_ToDeviceType: Unknown Button: %i", button);
    }
}

#[no_mangle]
pub extern "C" fn Button_ToString(button: Button) -> *const libc::c_char {
    match button {
        0 => c_str!("Button_Null"),
        1 => c_str!("Button_Keyboard_A"),
        2 => c_str!("Button_Keyboard_B"),
        3 => c_str!("Button_Keyboard_C"),
        4 => c_str!("Button_Keyboard_D"),
        5 => c_str!("Button_Keyboard_E"),
        6 => c_str!("Button_Keyboard_F"),
        7 => c_str!("Button_Keyboard_G"),
        8 => c_str!("Button_Keyboard_H"),
        9 => c_str!("Button_Keyboard_I"),
        10 => c_str!("Button_Keyboard_J"),
        11 => c_str!("Button_Keyboard_K"),
        12 => c_str!("Button_Keyboard_L"),
        13 => c_str!("Button_Keyboard_M"),
        14 => c_str!("Button_Keyboard_N"),
        15 => c_str!("Button_Keyboard_O"),
        16 => c_str!("Button_Keyboard_P"),
        17 => c_str!("Button_Keyboard_Q"),
        18 => c_str!("Button_Keyboard_R"),
        19 => c_str!("Button_Keyboard_S"),
        20 => c_str!("Button_Keyboard_T"),
        21 => c_str!("Button_Keyboard_U"),
        22 => c_str!("Button_Keyboard_V"),
        23 => c_str!("Button_Keyboard_W"),
        24 => c_str!("Button_Keyboard_X"),
        25 => c_str!("Button_Keyboard_Y"),
        26 => c_str!("Button_Keyboard_Z"),
        27 => c_str!("Button_Keyboard_N0"),
        28 => c_str!("Button_Keyboard_N1"),
        29 => c_str!("Button_Keyboard_N2"),
        30 => c_str!("Button_Keyboard_N3"),
        31 => c_str!("Button_Keyboard_N4"),
        32 => c_str!("Button_Keyboard_N5"),
        33 => c_str!("Button_Keyboard_N6"),
        34 => c_str!("Button_Keyboard_N7"),
        35 => c_str!("Button_Keyboard_N8"),
        36 => c_str!("Button_Keyboard_N9"),
        37 => c_str!("Button_Keyboard_F1"),
        38 => c_str!("Button_Keyboard_F2"),
        39 => c_str!("Button_Keyboard_F3"),
        40 => c_str!("Button_Keyboard_F4"),
        41 => c_str!("Button_Keyboard_F5"),
        42 => c_str!("Button_Keyboard_F6"),
        43 => c_str!("Button_Keyboard_F7"),
        44 => c_str!("Button_Keyboard_F8"),
        45 => c_str!("Button_Keyboard_F9"),
        46 => c_str!("Button_Keyboard_F10"),
        47 => c_str!("Button_Keyboard_F11"),
        48 => c_str!("Button_Keyboard_F12"),
        49 => c_str!("Button_Keyboard_F13"),
        50 => c_str!("Button_Keyboard_F14"),
        51 => c_str!("Button_Keyboard_F15"),
        52 => c_str!("Button_Keyboard_F16"),
        53 => c_str!("Button_Keyboard_F17"),
        54 => c_str!("Button_Keyboard_F18"),
        55 => c_str!("Button_Keyboard_F19"),
        56 => c_str!("Button_Keyboard_F20"),
        57 => c_str!("Button_Keyboard_F21"),
        58 => c_str!("Button_Keyboard_F22"),
        59 => c_str!("Button_Keyboard_F23"),
        60 => c_str!("Button_Keyboard_F24"),
        61 => c_str!("Button_Keyboard_KP0"),
        62 => c_str!("Button_Keyboard_KP1"),
        63 => c_str!("Button_Keyboard_KP2"),
        64 => c_str!("Button_Keyboard_KP3"),
        65 => c_str!("Button_Keyboard_KP4"),
        66 => c_str!("Button_Keyboard_KP5"),
        67 => c_str!("Button_Keyboard_KP6"),
        68 => c_str!("Button_Keyboard_KP7"),
        69 => c_str!("Button_Keyboard_KP8"),
        70 => c_str!("Button_Keyboard_KP9"),
        71 => c_str!("Button_Keyboard_KPNumLock"),
        72 => c_str!("Button_Keyboard_KPDivide"),
        73 => c_str!("Button_Keyboard_KPMultiply"),
        74 => c_str!("Button_Keyboard_KPSubtract"),
        75 => c_str!("Button_Keyboard_KPAdd"),
        76 => c_str!("Button_Keyboard_KPEnter"),
        77 => c_str!("Button_Keyboard_KPDecimal"),
        78 => c_str!("Button_Keyboard_Backspace"),
        79 => c_str!("Button_Keyboard_Escape"),
        80 => c_str!("Button_Keyboard_Return"),
        81 => c_str!("Button_Keyboard_Space"),
        82 => c_str!("Button_Keyboard_Tab"),
        83 => c_str!("Button_Keyboard_Backtick"),
        84 => c_str!("Button_Keyboard_CapsLock"),
        85 => c_str!("Button_Keyboard_Minus"),
        86 => c_str!("Button_Keyboard_Equals"),
        87 => c_str!("Button_Keyboard_LBracket"),
        88 => c_str!("Button_Keyboard_RBracket"),
        89 => c_str!("Button_Keyboard_Backslash"),
        90 => c_str!("Button_Keyboard_Semicolon"),
        91 => c_str!("Button_Keyboard_Apostrophe"),
        92 => c_str!("Button_Keyboard_Comma"),
        93 => c_str!("Button_Keyboard_Period"),
        94 => c_str!("Button_Keyboard_Slash"),
        95 => c_str!("Button_Keyboard_PrintScreen"),
        96 => c_str!("Button_Keyboard_ScrollLock"),
        97 => c_str!("Button_Keyboard_Pause"),
        98 => c_str!("Button_Keyboard_Insert"),
        99 => c_str!("Button_Keyboard_Delete"),
        100 => c_str!("Button_Keyboard_Home"),
        101 => c_str!("Button_Keyboard_End"),
        102 => c_str!("Button_Keyboard_PageUp"),
        103 => c_str!("Button_Keyboard_PageDown"),
        104 => c_str!("Button_Keyboard_Right"),
        105 => c_str!("Button_Keyboard_Left"),
        106 => c_str!("Button_Keyboard_Down"),
        107 => c_str!("Button_Keyboard_Up"),
        108 => c_str!("Button_Keyboard_LCtrl"),
        109 => c_str!("Button_Keyboard_LShift"),
        110 => c_str!("Button_Keyboard_LAlt"),
        111 => c_str!("Button_Keyboard_LMeta"),
        112 => c_str!("Button_Keyboard_RCtrl"),
        113 => c_str!("Button_Keyboard_RShift"),
        114 => c_str!("Button_Keyboard_RAlt"),
        115 => c_str!("Button_Keyboard_RMeta"),
        117 => c_str!("Button_Mouse_Left"),
        118 => c_str!("Button_Mouse_Middle"),
        119 => c_str!("Button_Mouse_Right"),
        120 => c_str!("Button_Mouse_X1"),
        121 => c_str!("Button_Mouse_X2"),
        122 => c_str!("Button_Mouse_X"),
        123 => c_str!("Button_Mouse_Y"),
        124 => c_str!("Button_Mouse_ScrollX"),
        125 => c_str!("Button_Mouse_ScrollY"),
        126 => c_str!("Button_Gamepad_A"),
        127 => c_str!("Button_Gamepad_B"),
        128 => c_str!("Button_Gamepad_X"),
        129 => c_str!("Button_Gamepad_Y"),
        130 => c_str!("Button_Gamepad_Back"),
        131 => c_str!("Button_Gamepad_Guide"),
        132 => c_str!("Button_Gamepad_Start"),
        133 => c_str!("Button_Gamepad_LStick"),
        134 => c_str!("Button_Gamepad_RStick"),
        135 => c_str!("Button_Gamepad_LBumper"),
        136 => c_str!("Button_Gamepad_RBumper"),
        137 => c_str!("Button_Gamepad_Up"),
        138 => c_str!("Button_Gamepad_Down"),
        139 => c_str!("Button_Gamepad_Left"),
        140 => c_str!("Button_Gamepad_Right"),
        141 => c_str!("Button_Gamepad_LTrigger"),
        142 => c_str!("Button_Gamepad_RTrigger"),
        143 => c_str!("Button_Gamepad_LStickX"),
        144 => c_str!("Button_Gamepad_LStickY"),
        145 => c_str!("Button_Gamepad_RStickX"),
        146 => c_str!("Button_Gamepad_RStickY"),
        147 => c_str!("Button_System_Exit"),
        _ if button == Button_System_Win_Enter => c_str!("Button_System_Win_Enter"),
        _ if button == Button_System_Win_Leave => c_str!("Button_System_Win_Leave"),
        _ => {
            ffi::StaticString!(format!("Unknown ({})", button))
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_IsAutoRelease(button: Button) -> bool {
    match button {
        b if b == Button_Mouse_ScrollX
            || b == Button_Mouse_ScrollY
            || b == Button_System_Exit
            || b == Button_System_Win_Enter
            || b == Button_System_Win_Leave =>
        {
            true
        }
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn Button_FromSDLScancode(scancode: SDL_Scancode) -> Button {
    /* NOTE : We don't support every possible scan code. */
    match scancode {
        SDL_Scancode::SDL_SCANCODE_UNKNOWN => Button_Null,
        SDL_Scancode::SDL_SCANCODE_A => Button_Keyboard_A,
        SDL_Scancode::SDL_SCANCODE_B => Button_Keyboard_B,
        SDL_Scancode::SDL_SCANCODE_C => Button_Keyboard_C,
        SDL_Scancode::SDL_SCANCODE_D => Button_Keyboard_D,
        SDL_Scancode::SDL_SCANCODE_E => Button_Keyboard_E,
        SDL_Scancode::SDL_SCANCODE_F => Button_Keyboard_F,
        SDL_Scancode::SDL_SCANCODE_G => Button_Keyboard_G,
        SDL_Scancode::SDL_SCANCODE_H => Button_Keyboard_H,
        SDL_Scancode::SDL_SCANCODE_I => Button_Keyboard_I,
        SDL_Scancode::SDL_SCANCODE_J => Button_Keyboard_J,
        SDL_Scancode::SDL_SCANCODE_K => Button_Keyboard_K,
        SDL_Scancode::SDL_SCANCODE_L => Button_Keyboard_L,
        SDL_Scancode::SDL_SCANCODE_M => Button_Keyboard_M,
        SDL_Scancode::SDL_SCANCODE_N => Button_Keyboard_N,
        SDL_Scancode::SDL_SCANCODE_O => Button_Keyboard_O,
        SDL_Scancode::SDL_SCANCODE_P => Button_Keyboard_P,
        SDL_Scancode::SDL_SCANCODE_Q => Button_Keyboard_Q,
        SDL_Scancode::SDL_SCANCODE_R => Button_Keyboard_R,
        SDL_Scancode::SDL_SCANCODE_S => Button_Keyboard_S,
        SDL_Scancode::SDL_SCANCODE_T => Button_Keyboard_T,
        SDL_Scancode::SDL_SCANCODE_U => Button_Keyboard_U,
        SDL_Scancode::SDL_SCANCODE_V => Button_Keyboard_V,
        SDL_Scancode::SDL_SCANCODE_W => Button_Keyboard_W,
        SDL_Scancode::SDL_SCANCODE_X => Button_Keyboard_X,
        SDL_Scancode::SDL_SCANCODE_Y => Button_Keyboard_Y,
        SDL_Scancode::SDL_SCANCODE_Z => Button_Keyboard_Z,
        SDL_Scancode::SDL_SCANCODE_0 => Button_Keyboard_N0,
        SDL_Scancode::SDL_SCANCODE_1 => Button_Keyboard_N1,
        SDL_Scancode::SDL_SCANCODE_2 => Button_Keyboard_N2,
        SDL_Scancode::SDL_SCANCODE_3 => Button_Keyboard_N3,
        SDL_Scancode::SDL_SCANCODE_4 => Button_Keyboard_N4,
        SDL_Scancode::SDL_SCANCODE_5 => Button_Keyboard_N5,
        SDL_Scancode::SDL_SCANCODE_6 => Button_Keyboard_N6,
        SDL_Scancode::SDL_SCANCODE_7 => Button_Keyboard_N7,
        SDL_Scancode::SDL_SCANCODE_8 => Button_Keyboard_N8,
        SDL_Scancode::SDL_SCANCODE_9 => Button_Keyboard_N9,
        SDL_Scancode::SDL_SCANCODE_F1 => Button_Keyboard_F1,
        SDL_Scancode::SDL_SCANCODE_F2 => Button_Keyboard_F2,
        SDL_Scancode::SDL_SCANCODE_F3 => Button_Keyboard_F3,
        SDL_Scancode::SDL_SCANCODE_F4 => Button_Keyboard_F4,
        SDL_Scancode::SDL_SCANCODE_F5 => Button_Keyboard_F5,
        SDL_Scancode::SDL_SCANCODE_F6 => Button_Keyboard_F6,
        SDL_Scancode::SDL_SCANCODE_F7 => Button_Keyboard_F7,
        SDL_Scancode::SDL_SCANCODE_F8 => Button_Keyboard_F8,
        SDL_Scancode::SDL_SCANCODE_F9 => Button_Keyboard_F9,
        SDL_Scancode::SDL_SCANCODE_F10 => Button_Keyboard_F10,
        SDL_Scancode::SDL_SCANCODE_F11 => Button_Keyboard_F11,
        SDL_Scancode::SDL_SCANCODE_F12 => Button_Keyboard_F12,
        SDL_Scancode::SDL_SCANCODE_F13 => Button_Keyboard_F13,
        SDL_Scancode::SDL_SCANCODE_F14 => Button_Keyboard_F14,
        SDL_Scancode::SDL_SCANCODE_F15 => Button_Keyboard_F15,
        SDL_Scancode::SDL_SCANCODE_F16 => Button_Keyboard_F16,
        SDL_Scancode::SDL_SCANCODE_F17 => Button_Keyboard_F17,
        SDL_Scancode::SDL_SCANCODE_F18 => Button_Keyboard_F18,
        SDL_Scancode::SDL_SCANCODE_F19 => Button_Keyboard_F19,
        SDL_Scancode::SDL_SCANCODE_F20 => Button_Keyboard_F20,
        SDL_Scancode::SDL_SCANCODE_F21 => Button_Keyboard_F21,
        SDL_Scancode::SDL_SCANCODE_F22 => Button_Keyboard_F22,
        SDL_Scancode::SDL_SCANCODE_F23 => Button_Keyboard_F23,
        SDL_Scancode::SDL_SCANCODE_F24 => Button_Keyboard_F24,
        SDL_Scancode::SDL_SCANCODE_KP_0 => Button_Keyboard_KP0,
        SDL_Scancode::SDL_SCANCODE_KP_1 => Button_Keyboard_KP1,
        SDL_Scancode::SDL_SCANCODE_KP_2 => Button_Keyboard_KP2,
        SDL_Scancode::SDL_SCANCODE_KP_3 => Button_Keyboard_KP3,
        SDL_Scancode::SDL_SCANCODE_KP_4 => Button_Keyboard_KP4,
        SDL_Scancode::SDL_SCANCODE_KP_5 => Button_Keyboard_KP5,
        SDL_Scancode::SDL_SCANCODE_KP_6 => Button_Keyboard_KP6,
        SDL_Scancode::SDL_SCANCODE_KP_7 => Button_Keyboard_KP7,
        SDL_Scancode::SDL_SCANCODE_KP_8 => Button_Keyboard_KP8,
        SDL_Scancode::SDL_SCANCODE_KP_9 => Button_Keyboard_KP9,
        SDL_Scancode::SDL_SCANCODE_NUMLOCKCLEAR => Button_Keyboard_KPNumLock,
        SDL_Scancode::SDL_SCANCODE_KP_DIVIDE => Button_Keyboard_KPDivide,
        SDL_Scancode::SDL_SCANCODE_KP_MULTIPLY => Button_Keyboard_KPMultiply,
        SDL_Scancode::SDL_SCANCODE_KP_MINUS => Button_Keyboard_KPSubtract,
        SDL_Scancode::SDL_SCANCODE_KP_PLUS => Button_Keyboard_KPAdd,
        SDL_Scancode::SDL_SCANCODE_KP_ENTER => Button_Keyboard_KPEnter,
        SDL_Scancode::SDL_SCANCODE_KP_DECIMAL => Button_Keyboard_KPDecimal,
        SDL_Scancode::SDL_SCANCODE_BACKSPACE => Button_Keyboard_Backspace,
        SDL_Scancode::SDL_SCANCODE_ESCAPE => Button_Keyboard_Escape,
        SDL_Scancode::SDL_SCANCODE_RETURN => Button_Keyboard_Return,
        SDL_Scancode::SDL_SCANCODE_SPACE => Button_Keyboard_Space,
        SDL_Scancode::SDL_SCANCODE_TAB => Button_Keyboard_Tab,
        SDL_Scancode::SDL_SCANCODE_GRAVE => Button_Keyboard_Backtick,
        SDL_Scancode::SDL_SCANCODE_CAPSLOCK => Button_Keyboard_CapsLock,
        SDL_Scancode::SDL_SCANCODE_MINUS => Button_Keyboard_Minus,
        SDL_Scancode::SDL_SCANCODE_EQUALS => Button_Keyboard_Equals,
        SDL_Scancode::SDL_SCANCODE_LEFTBRACKET => Button_Keyboard_LBracket,
        SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET => Button_Keyboard_RBracket,
        SDL_Scancode::SDL_SCANCODE_BACKSLASH => Button_Keyboard_Backslash,
        SDL_Scancode::SDL_SCANCODE_SEMICOLON => Button_Keyboard_Semicolon,
        SDL_Scancode::SDL_SCANCODE_APOSTROPHE => Button_Keyboard_Apostrophe,
        SDL_Scancode::SDL_SCANCODE_COMMA => Button_Keyboard_Comma,
        SDL_Scancode::SDL_SCANCODE_PERIOD => Button_Keyboard_Period,
        SDL_Scancode::SDL_SCANCODE_SLASH => Button_Keyboard_Slash,
        SDL_Scancode::SDL_SCANCODE_PRINTSCREEN => Button_Keyboard_PrintScreen,
        SDL_Scancode::SDL_SCANCODE_SCROLLLOCK => Button_Keyboard_ScrollLock,
        SDL_Scancode::SDL_SCANCODE_PAUSE => Button_Keyboard_Pause,
        SDL_Scancode::SDL_SCANCODE_INSERT => Button_Keyboard_Insert,
        SDL_Scancode::SDL_SCANCODE_DELETE => Button_Keyboard_Delete,
        SDL_Scancode::SDL_SCANCODE_HOME => Button_Keyboard_Home,
        SDL_Scancode::SDL_SCANCODE_END => Button_Keyboard_End,
        SDL_Scancode::SDL_SCANCODE_PAGEUP => Button_Keyboard_PageUp,
        SDL_Scancode::SDL_SCANCODE_PAGEDOWN => Button_Keyboard_PageDown,
        SDL_Scancode::SDL_SCANCODE_RIGHT => Button_Keyboard_Right,
        SDL_Scancode::SDL_SCANCODE_LEFT => Button_Keyboard_Left,
        SDL_Scancode::SDL_SCANCODE_DOWN => Button_Keyboard_Down,
        SDL_Scancode::SDL_SCANCODE_UP => Button_Keyboard_Up,
        SDL_Scancode::SDL_SCANCODE_LCTRL => Button_Keyboard_LCtrl,
        SDL_Scancode::SDL_SCANCODE_LSHIFT => Button_Keyboard_LShift,
        SDL_Scancode::SDL_SCANCODE_LALT => Button_Keyboard_LAlt,
        SDL_Scancode::SDL_SCANCODE_LGUI => Button_Keyboard_LMeta,
        SDL_Scancode::SDL_SCANCODE_RCTRL => Button_Keyboard_RCtrl,
        SDL_Scancode::SDL_SCANCODE_RSHIFT => Button_Keyboard_RShift,
        SDL_Scancode::SDL_SCANCODE_RALT => Button_Keyboard_RAlt,
        SDL_Scancode::SDL_SCANCODE_RGUI => Button_Keyboard_RMeta,
        _ => Button_Null,
    }
}

#[no_mangle]
pub extern "C" fn Button_ToSDLScancode(button: Button) -> SDL_Scancode {
    match button {
        b if b == Button_Null => SDL_Scancode::SDL_SCANCODE_UNKNOWN,
        b if b == Button_Keyboard_A => SDL_Scancode::SDL_SCANCODE_A,
        b if b == Button_Keyboard_B => SDL_Scancode::SDL_SCANCODE_B,
        b if b == Button_Keyboard_C => SDL_Scancode::SDL_SCANCODE_C,
        b if b == Button_Keyboard_D => SDL_Scancode::SDL_SCANCODE_D,
        b if b == Button_Keyboard_E => SDL_Scancode::SDL_SCANCODE_E,
        b if b == Button_Keyboard_F => SDL_Scancode::SDL_SCANCODE_F,
        b if b == Button_Keyboard_G => SDL_Scancode::SDL_SCANCODE_G,
        b if b == Button_Keyboard_H => SDL_Scancode::SDL_SCANCODE_H,
        b if b == Button_Keyboard_I => SDL_Scancode::SDL_SCANCODE_I,
        b if b == Button_Keyboard_J => SDL_Scancode::SDL_SCANCODE_J,
        b if b == Button_Keyboard_K => SDL_Scancode::SDL_SCANCODE_K,
        b if b == Button_Keyboard_L => SDL_Scancode::SDL_SCANCODE_L,
        b if b == Button_Keyboard_M => SDL_Scancode::SDL_SCANCODE_M,
        b if b == Button_Keyboard_N => SDL_Scancode::SDL_SCANCODE_N,
        b if b == Button_Keyboard_O => SDL_Scancode::SDL_SCANCODE_O,
        b if b == Button_Keyboard_P => SDL_Scancode::SDL_SCANCODE_P,
        b if b == Button_Keyboard_Q => SDL_Scancode::SDL_SCANCODE_Q,
        b if b == Button_Keyboard_R => SDL_Scancode::SDL_SCANCODE_R,
        b if b == Button_Keyboard_S => SDL_Scancode::SDL_SCANCODE_S,
        b if b == Button_Keyboard_T => SDL_Scancode::SDL_SCANCODE_T,
        b if b == Button_Keyboard_U => SDL_Scancode::SDL_SCANCODE_U,
        b if b == Button_Keyboard_V => SDL_Scancode::SDL_SCANCODE_V,
        b if b == Button_Keyboard_W => SDL_Scancode::SDL_SCANCODE_W,
        b if b == Button_Keyboard_X => SDL_Scancode::SDL_SCANCODE_X,
        b if b == Button_Keyboard_Y => SDL_Scancode::SDL_SCANCODE_Y,
        b if b == Button_Keyboard_Z => SDL_Scancode::SDL_SCANCODE_Z,
        b if b == Button_Keyboard_N0 => SDL_Scancode::SDL_SCANCODE_0,
        b if b == Button_Keyboard_N1 => SDL_Scancode::SDL_SCANCODE_1,
        b if b == Button_Keyboard_N2 => SDL_Scancode::SDL_SCANCODE_2,
        b if b == Button_Keyboard_N3 => SDL_Scancode::SDL_SCANCODE_3,
        b if b == Button_Keyboard_N4 => SDL_Scancode::SDL_SCANCODE_4,
        b if b == Button_Keyboard_N5 => SDL_Scancode::SDL_SCANCODE_5,
        b if b == Button_Keyboard_N6 => SDL_Scancode::SDL_SCANCODE_6,
        b if b == Button_Keyboard_N7 => SDL_Scancode::SDL_SCANCODE_7,
        b if b == Button_Keyboard_N8 => SDL_Scancode::SDL_SCANCODE_8,
        b if b == Button_Keyboard_N9 => SDL_Scancode::SDL_SCANCODE_9,
        b if b == Button_Keyboard_F1 => SDL_Scancode::SDL_SCANCODE_F1,
        b if b == Button_Keyboard_F2 => SDL_Scancode::SDL_SCANCODE_F2,
        b if b == Button_Keyboard_F3 => SDL_Scancode::SDL_SCANCODE_F3,
        b if b == Button_Keyboard_F4 => SDL_Scancode::SDL_SCANCODE_F4,
        b if b == Button_Keyboard_F5 => SDL_Scancode::SDL_SCANCODE_F5,
        b if b == Button_Keyboard_F6 => SDL_Scancode::SDL_SCANCODE_F6,
        b if b == Button_Keyboard_F7 => SDL_Scancode::SDL_SCANCODE_F7,
        b if b == Button_Keyboard_F8 => SDL_Scancode::SDL_SCANCODE_F8,
        b if b == Button_Keyboard_F9 => SDL_Scancode::SDL_SCANCODE_F9,
        b if b == Button_Keyboard_F10 => SDL_Scancode::SDL_SCANCODE_F10,
        b if b == Button_Keyboard_F11 => SDL_Scancode::SDL_SCANCODE_F11,
        b if b == Button_Keyboard_F12 => SDL_Scancode::SDL_SCANCODE_F12,
        b if b == Button_Keyboard_F13 => SDL_Scancode::SDL_SCANCODE_F13,
        b if b == Button_Keyboard_F14 => SDL_Scancode::SDL_SCANCODE_F14,
        b if b == Button_Keyboard_F15 => SDL_Scancode::SDL_SCANCODE_F15,
        b if b == Button_Keyboard_F16 => SDL_Scancode::SDL_SCANCODE_F16,
        b if b == Button_Keyboard_F17 => SDL_Scancode::SDL_SCANCODE_F17,
        b if b == Button_Keyboard_F18 => SDL_Scancode::SDL_SCANCODE_F18,
        b if b == Button_Keyboard_F19 => SDL_Scancode::SDL_SCANCODE_F19,
        b if b == Button_Keyboard_F20 => SDL_Scancode::SDL_SCANCODE_F20,
        b if b == Button_Keyboard_F21 => SDL_Scancode::SDL_SCANCODE_F21,
        b if b == Button_Keyboard_F22 => SDL_Scancode::SDL_SCANCODE_F22,
        b if b == Button_Keyboard_F23 => SDL_Scancode::SDL_SCANCODE_F23,
        b if b == Button_Keyboard_F24 => SDL_Scancode::SDL_SCANCODE_F24,
        b if b == Button_Keyboard_KP0 => SDL_Scancode::SDL_SCANCODE_KP_0,
        b if b == Button_Keyboard_KP1 => SDL_Scancode::SDL_SCANCODE_KP_1,
        b if b == Button_Keyboard_KP2 => SDL_Scancode::SDL_SCANCODE_KP_2,
        b if b == Button_Keyboard_KP3 => SDL_Scancode::SDL_SCANCODE_KP_3,
        b if b == Button_Keyboard_KP4 => SDL_Scancode::SDL_SCANCODE_KP_4,
        b if b == Button_Keyboard_KP5 => SDL_Scancode::SDL_SCANCODE_KP_5,
        b if b == Button_Keyboard_KP6 => SDL_Scancode::SDL_SCANCODE_KP_6,
        b if b == Button_Keyboard_KP7 => SDL_Scancode::SDL_SCANCODE_KP_7,
        b if b == Button_Keyboard_KP8 => SDL_Scancode::SDL_SCANCODE_KP_8,
        b if b == Button_Keyboard_KP9 => SDL_Scancode::SDL_SCANCODE_KP_9,
        b if b == Button_Keyboard_KPNumLock => SDL_Scancode::SDL_SCANCODE_NUMLOCKCLEAR,
        b if b == Button_Keyboard_KPDivide => SDL_Scancode::SDL_SCANCODE_KP_DIVIDE,
        b if b == Button_Keyboard_KPMultiply => SDL_Scancode::SDL_SCANCODE_KP_MULTIPLY,
        b if b == Button_Keyboard_KPSubtract => SDL_Scancode::SDL_SCANCODE_KP_MINUS,
        b if b == Button_Keyboard_KPAdd => SDL_Scancode::SDL_SCANCODE_KP_PLUS,
        b if b == Button_Keyboard_KPEnter => SDL_Scancode::SDL_SCANCODE_KP_ENTER,
        b if b == Button_Keyboard_KPDecimal => SDL_Scancode::SDL_SCANCODE_KP_DECIMAL,
        b if b == Button_Keyboard_Backspace => SDL_Scancode::SDL_SCANCODE_BACKSPACE,
        b if b == Button_Keyboard_Escape => SDL_Scancode::SDL_SCANCODE_ESCAPE,
        b if b == Button_Keyboard_Return => SDL_Scancode::SDL_SCANCODE_RETURN,
        b if b == Button_Keyboard_Space => SDL_Scancode::SDL_SCANCODE_SPACE,
        b if b == Button_Keyboard_Tab => SDL_Scancode::SDL_SCANCODE_TAB,
        b if b == Button_Keyboard_Backtick => SDL_Scancode::SDL_SCANCODE_GRAVE,
        b if b == Button_Keyboard_CapsLock => SDL_Scancode::SDL_SCANCODE_CAPSLOCK,
        b if b == Button_Keyboard_Minus => SDL_Scancode::SDL_SCANCODE_MINUS,
        b if b == Button_Keyboard_Equals => SDL_Scancode::SDL_SCANCODE_EQUALS,
        b if b == Button_Keyboard_LBracket => SDL_Scancode::SDL_SCANCODE_LEFTBRACKET,
        b if b == Button_Keyboard_RBracket => SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET,
        b if b == Button_Keyboard_Backslash => SDL_Scancode::SDL_SCANCODE_BACKSLASH,
        b if b == Button_Keyboard_Semicolon => SDL_Scancode::SDL_SCANCODE_SEMICOLON,
        b if b == Button_Keyboard_Apostrophe => SDL_Scancode::SDL_SCANCODE_APOSTROPHE,
        b if b == Button_Keyboard_Comma => SDL_Scancode::SDL_SCANCODE_COMMA,
        b if b == Button_Keyboard_Period => SDL_Scancode::SDL_SCANCODE_PERIOD,
        b if b == Button_Keyboard_Slash => SDL_Scancode::SDL_SCANCODE_SLASH,
        b if b == Button_Keyboard_PrintScreen => SDL_Scancode::SDL_SCANCODE_PRINTSCREEN,
        b if b == Button_Keyboard_ScrollLock => SDL_Scancode::SDL_SCANCODE_SCROLLLOCK,
        b if b == Button_Keyboard_Pause => SDL_Scancode::SDL_SCANCODE_PAUSE,
        b if b == Button_Keyboard_Insert => SDL_Scancode::SDL_SCANCODE_INSERT,
        b if b == Button_Keyboard_Delete => SDL_Scancode::SDL_SCANCODE_DELETE,
        b if b == Button_Keyboard_Home => SDL_Scancode::SDL_SCANCODE_HOME,
        b if b == Button_Keyboard_End => SDL_Scancode::SDL_SCANCODE_END,
        b if b == Button_Keyboard_PageUp => SDL_Scancode::SDL_SCANCODE_PAGEUP,
        b if b == Button_Keyboard_PageDown => SDL_Scancode::SDL_SCANCODE_PAGEDOWN,
        b if b == Button_Keyboard_Right => SDL_Scancode::SDL_SCANCODE_RIGHT,
        b if b == Button_Keyboard_Left => SDL_Scancode::SDL_SCANCODE_LEFT,
        b if b == Button_Keyboard_Down => SDL_Scancode::SDL_SCANCODE_DOWN,
        b if b == Button_Keyboard_Up => SDL_Scancode::SDL_SCANCODE_UP,
        b if b == Button_Keyboard_LCtrl => SDL_Scancode::SDL_SCANCODE_LCTRL,
        b if b == Button_Keyboard_LShift => SDL_Scancode::SDL_SCANCODE_LSHIFT,
        b if b == Button_Keyboard_LAlt => SDL_Scancode::SDL_SCANCODE_LALT,
        b if b == Button_Keyboard_LMeta => SDL_Scancode::SDL_SCANCODE_LGUI,
        b if b == Button_Keyboard_RCtrl => SDL_Scancode::SDL_SCANCODE_RCTRL,
        b if b == Button_Keyboard_RShift => SDL_Scancode::SDL_SCANCODE_RSHIFT,
        b if b == Button_Keyboard_RAlt => SDL_Scancode::SDL_SCANCODE_RALT,
        b if b == Button_Keyboard_RMeta => SDL_Scancode::SDL_SCANCODE_RGUI,
        _ => {
            CFatal!("Button_ToSDLScancode: Unhandled case: %i", button);
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_FromSDLMouseButton(mouseButton: u8) -> Button {
    match mouseButton as u32 {
        mb if mb == SDL_BUTTON_LEFT => Button_Mouse_Left,
        mb if mb == SDL_BUTTON_MIDDLE => Button_Mouse_Middle,
        mb if mb == SDL_BUTTON_RIGHT => Button_Mouse_Right,
        mb if mb == SDL_BUTTON_X1 => Button_Mouse_X1,
        mb if mb == SDL_BUTTON_X2 => Button_Mouse_X2,
        _ => {
            CFatal!(
                "Button_FromSDLMouseButton: Unhandled case: %i",
                mouseButton as i32,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_ToSDLMouseButton(button: Button) -> u8 {
    match button {
        mb if mb == Button_Mouse_Left => SDL_BUTTON_LEFT as u8,
        mb if mb == Button_Mouse_Middle => SDL_BUTTON_MIDDLE as u8,
        mb if mb == Button_Mouse_Right => SDL_BUTTON_RIGHT as u8,
        mb if mb == Button_Mouse_X1 => SDL_BUTTON_X1 as u8,
        mb if mb == Button_Mouse_X2 => SDL_BUTTON_X2 as u8,
        _ => {
            CFatal!("Button_ToSDLMouseButton: Unhandled case: %i", button,);
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_FromSDLControllerAxis(controllerAxis: SDL_GameControllerAxis) -> Button {
    match controllerAxis {
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX => Button_Gamepad_LStickX,
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY => Button_Gamepad_LStickY,
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX => Button_Gamepad_RStickX,
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY => Button_Gamepad_RStickY,
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT => Button_Gamepad_LTrigger,
        SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT => Button_Gamepad_RTrigger,
        _ => {
            CFatal!(
                "Button_FromSDLControllerAxis: Unhandled case: %i",
                controllerAxis as i32,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_ToSDLControllerAxis(button: Button) -> SDL_GameControllerAxis {
    match button {
        b if b == Button_Gamepad_LStickX => SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX,
        b if b == Button_Gamepad_LStickY => SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY,
        b if b == Button_Gamepad_RStickX => SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX,
        b if b == Button_Gamepad_RStickY => SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY,
        b if b == Button_Gamepad_LTrigger => {
            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT
        }
        b if b == Button_Gamepad_RTrigger => {
            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT
        }
        _ => {
            CFatal!("Button_ToSDLControllerAxis: Unhandled case: %i", button,);
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_FromSDLControllerButton(
    controllerButton: SDL_GameControllerButton,
) -> Button {
    match controllerButton {
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A => Button_Gamepad_A,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B => Button_Gamepad_B,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X => Button_Gamepad_X,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y => Button_Gamepad_Y,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK => Button_Gamepad_Back,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_GUIDE => Button_Gamepad_Guide,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START => Button_Gamepad_Start,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK => Button_Gamepad_LStick,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK => Button_Gamepad_RStick,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER => Button_Gamepad_LBumper,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER => Button_Gamepad_RBumper,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP => Button_Gamepad_Up,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN => Button_Gamepad_Down,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT => Button_Gamepad_Left,
        SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT => Button_Gamepad_Right,
        _ => {
            CFatal!(
                "Button_FromSDLControllerButton: Unhandled case: %i",
                controllerButton as i32,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn Button_ToSDLControllerButton(button: Button) -> SDL_GameControllerButton {
    match button {
        b if b == Button_Gamepad_A => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A,
        b if b == Button_Gamepad_B => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B,
        b if b == Button_Gamepad_X => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X,
        b if b == Button_Gamepad_Y => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y,
        b if b == Button_Gamepad_Back => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK,
        b if b == Button_Gamepad_Guide => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_GUIDE,
        b if b == Button_Gamepad_Start => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START,
        b if b == Button_Gamepad_LStick => {
            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK
        }
        b if b == Button_Gamepad_RStick => {
            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK
        }
        b if b == Button_Gamepad_LBumper => {
            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER
        }
        b if b == Button_Gamepad_RBumper => {
            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER
        }
        b if b == Button_Gamepad_Up => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP,
        b if b == Button_Gamepad_Down => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN,
        b if b == Button_Gamepad_Left => SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT,
        b if b == Button_Gamepad_Right => {
            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT
        }
        _ => {
            CFatal!("Button_ToSDLControllerButton: Unhandled case: %i", button,);
        }
    }
}
