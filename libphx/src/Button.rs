use crate::internal::Memory::*;
use crate::Common::*;
use crate::DeviceType::*;
use libc;
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
pub static Button_System_Last: Button = Button_System_First + 0;

#[no_mangle]
pub static Button_Last: Button = Button_System_Last;

#[no_mangle]
pub unsafe extern "C" fn Button_ToDeviceType(button: Button) -> DeviceType {
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
        Fatal(c_str!("Button_ToDeviceType: Unknown Button: %i"), button)
    }
}

#[no_mangle]
pub unsafe extern "C" fn Button_ToString(button: Button) -> *const libc::c_char {
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
        _ => {
            static mut buffer: [libc::c_char; 512] = [0; 512];
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                c_str!("Unknown (%i)"),
                button,
            );
            buffer.as_mut_ptr() as *const libc::c_char
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Button_IsAutoRelease(button: Button) -> bool {
    match button {
        124 | 125 | 147 => true,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLScancode(scancode: SDL_Scancode) -> Button {
    match scancode as u32 {
        0 => Button_Null,
        4 => Button_Keyboard_A,
        5 => Button_Keyboard_B,
        6 => Button_Keyboard_C,
        7 => Button_Keyboard_D,
        8 => Button_Keyboard_E,
        9 => Button_Keyboard_F,
        10 => Button_Keyboard_G,
        11 => Button_Keyboard_H,
        12 => Button_Keyboard_I,
        13 => Button_Keyboard_J,
        14 => Button_Keyboard_K,
        15 => Button_Keyboard_L,
        16 => Button_Keyboard_M,
        17 => Button_Keyboard_N,
        18 => Button_Keyboard_O,
        19 => Button_Keyboard_P,
        20 => Button_Keyboard_Q,
        21 => Button_Keyboard_R,
        22 => Button_Keyboard_S,
        23 => Button_Keyboard_T,
        24 => Button_Keyboard_U,
        25 => Button_Keyboard_V,
        26 => Button_Keyboard_W,
        27 => Button_Keyboard_X,
        28 => Button_Keyboard_Y,
        29 => Button_Keyboard_Z,
        39 => Button_Keyboard_N0,
        30 => Button_Keyboard_N1,
        31 => Button_Keyboard_N2,
        32 => Button_Keyboard_N3,
        33 => Button_Keyboard_N4,
        34 => Button_Keyboard_N5,
        35 => Button_Keyboard_N6,
        36 => Button_Keyboard_N7,
        37 => Button_Keyboard_N8,
        38 => Button_Keyboard_N9,
        58 => Button_Keyboard_F1,
        59 => Button_Keyboard_F2,
        60 => Button_Keyboard_F3,
        61 => Button_Keyboard_F4,
        62 => Button_Keyboard_F5,
        63 => Button_Keyboard_F6,
        64 => Button_Keyboard_F7,
        65 => Button_Keyboard_F8,
        66 => Button_Keyboard_F9,
        67 => Button_Keyboard_F10,
        68 => Button_Keyboard_F11,
        69 => Button_Keyboard_F12,
        104 => Button_Keyboard_F13,
        105 => Button_Keyboard_F14,
        106 => Button_Keyboard_F15,
        107 => Button_Keyboard_F16,
        108 => Button_Keyboard_F17,
        109 => Button_Keyboard_F18,
        110 => Button_Keyboard_F19,
        111 => Button_Keyboard_F20,
        112 => Button_Keyboard_F21,
        113 => Button_Keyboard_F22,
        114 => Button_Keyboard_F23,
        115 => Button_Keyboard_F24,
        98 => Button_Keyboard_KP0,
        89 => Button_Keyboard_KP1,
        90 => Button_Keyboard_KP2,
        91 => Button_Keyboard_KP3,
        92 => Button_Keyboard_KP4,
        93 => Button_Keyboard_KP5,
        94 => Button_Keyboard_KP6,
        95 => Button_Keyboard_KP7,
        96 => Button_Keyboard_KP8,
        97 => Button_Keyboard_KP9,
        83 => Button_Keyboard_KPNumLock,
        84 => Button_Keyboard_KPDivide,
        85 => Button_Keyboard_KPMultiply,
        86 => Button_Keyboard_KPSubtract,
        87 => Button_Keyboard_KPAdd,
        88 => Button_Keyboard_KPEnter,
        220 => Button_Keyboard_KPDecimal,
        42 => Button_Keyboard_Backspace,
        41 => Button_Keyboard_Escape,
        40 => Button_Keyboard_Return,
        44 => Button_Keyboard_Space,
        43 => Button_Keyboard_Tab,
        53 => Button_Keyboard_Backtick,
        57 => Button_Keyboard_CapsLock,
        45 => Button_Keyboard_Minus,
        46 => Button_Keyboard_Equals,
        47 => Button_Keyboard_LBracket,
        48 => Button_Keyboard_RBracket,
        49 => Button_Keyboard_Backslash,
        51 => Button_Keyboard_Semicolon,
        52 => Button_Keyboard_Apostrophe,
        54 => Button_Keyboard_Comma,
        55 => Button_Keyboard_Period,
        56 => Button_Keyboard_Slash,
        70 => Button_Keyboard_PrintScreen,
        71 => Button_Keyboard_ScrollLock,
        72 => Button_Keyboard_Pause,
        73 => Button_Keyboard_Insert,
        76 => Button_Keyboard_Delete,
        74 => Button_Keyboard_Home,
        77 => Button_Keyboard_End,
        75 => Button_Keyboard_PageUp,
        78 => Button_Keyboard_PageDown,
        79 => Button_Keyboard_Right,
        80 => Button_Keyboard_Left,
        81 => Button_Keyboard_Down,
        82 => Button_Keyboard_Up,
        224 => Button_Keyboard_LCtrl,
        225 => Button_Keyboard_LShift,
        226 => Button_Keyboard_LAlt,
        227 => Button_Keyboard_LMeta,
        228 => Button_Keyboard_RCtrl,
        229 => Button_Keyboard_RShift,
        230 => Button_Keyboard_RAlt,
        231 => Button_Keyboard_RMeta,
        _ => Button_Null,
    }
}

#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLScancode(button: Button) -> SDL_Scancode {
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
            Fatal(c_str!("Button_ToSDLScancode: Unhandled case: %i"), button);
        }
    }
    SDL_Scancode::SDL_SCANCODE_UNKNOWN
}

#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLMouseButton(mouseButton: u8) -> Button {
    match mouseButton as i32 {
        1 => {}
        2 => return Button_Mouse_Middle,
        3 => return Button_Mouse_Right,
        4 => return Button_Mouse_X1,
        5 => return Button_Mouse_X2,
        _ => {
            Fatal(
                c_str!("Button_FromSDLMouseButton: Unhandled case: %i"),
                mouseButton as i32,
            );
        }
    }
    Button_Mouse_Left
}

#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLMouseButton(button: Button) -> u8 {
    match button {
        117 => {}
        118 => return 2 as u8,
        119 => return 3 as u8,
        120 => return 4 as u8,
        121 => return 5 as u8,
        _ => {
            Fatal(
                c_str!("Button_ToSDLMouseButton: Unhandled case: %i"),
                button,
            );
        }
    }
    1 as u8
}

#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerAxis(
    controllerAxis: SDL_GameControllerAxis,
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
                c_str!("Button_FromSDLControllerAxis: Unhandled case: %i"),
                controllerAxis as i32,
            );
        }
    }
    Button_Gamepad_LStickX
}

#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLControllerAxis(button: Button) -> SDL_GameControllerAxis {
    match button {
        143 => {}
        144 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY,
        145 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX,
        146 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY,
        141 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT,
        142 => return SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT,
        _ => {
            Fatal(
                c_str!("Button_ToSDLControllerAxis: Unhandled case: %i"),
                button,
            );
        }
    }
    SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX
}

#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerButton(
    controllerButton: SDL_GameControllerButton,
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
                c_str!("Button_FromSDLControllerButton: Unhandled case: %i"),
                controllerButton as i32,
            );
        }
    }
    Button_Gamepad_A
}

#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLControllerButton(button: Button) -> SDL_GameControllerButton {
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
                c_str!("Button_ToSDLControllerButton: Unhandled case: %i"),
                button,
            );
        }
    }
    SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A
}
