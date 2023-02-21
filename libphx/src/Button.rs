use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
}
pub type cstr = *const libc::c_char;
pub type Button = i32;
pub type DeviceType = i32;
pub type SDL_Scancode = libc::c_uint;
pub const SDL_NUM_SCANCODES: SDL_Scancode = 512;
pub const SDL_SCANCODE_ENDCALL: SDL_Scancode = 290;
pub const SDL_SCANCODE_CALL: SDL_Scancode = 289;
pub const SDL_SCANCODE_SOFTRIGHT: SDL_Scancode = 288;
pub const SDL_SCANCODE_SOFTLEFT: SDL_Scancode = 287;
pub const SDL_SCANCODE_AUDIOFASTFORWARD: SDL_Scancode = 286;
pub const SDL_SCANCODE_AUDIOREWIND: SDL_Scancode = 285;
pub const SDL_SCANCODE_APP2: SDL_Scancode = 284;
pub const SDL_SCANCODE_APP1: SDL_Scancode = 283;
pub const SDL_SCANCODE_SLEEP: SDL_Scancode = 282;
pub const SDL_SCANCODE_EJECT: SDL_Scancode = 281;
pub const SDL_SCANCODE_KBDILLUMUP: SDL_Scancode = 280;
pub const SDL_SCANCODE_KBDILLUMDOWN: SDL_Scancode = 279;
pub const SDL_SCANCODE_KBDILLUMTOGGLE: SDL_Scancode = 278;
pub const SDL_SCANCODE_DISPLAYSWITCH: SDL_Scancode = 277;
pub const SDL_SCANCODE_BRIGHTNESSUP: SDL_Scancode = 276;
pub const SDL_SCANCODE_BRIGHTNESSDOWN: SDL_Scancode = 275;
pub const SDL_SCANCODE_AC_BOOKMARKS: SDL_Scancode = 274;
pub const SDL_SCANCODE_AC_REFRESH: SDL_Scancode = 273;
pub const SDL_SCANCODE_AC_STOP: SDL_Scancode = 272;
pub const SDL_SCANCODE_AC_FORWARD: SDL_Scancode = 271;
pub const SDL_SCANCODE_AC_BACK: SDL_Scancode = 270;
pub const SDL_SCANCODE_AC_HOME: SDL_Scancode = 269;
pub const SDL_SCANCODE_AC_SEARCH: SDL_Scancode = 268;
pub const SDL_SCANCODE_COMPUTER: SDL_Scancode = 267;
pub const SDL_SCANCODE_CALCULATOR: SDL_Scancode = 266;
pub const SDL_SCANCODE_MAIL: SDL_Scancode = 265;
pub const SDL_SCANCODE_WWW: SDL_Scancode = 264;
pub const SDL_SCANCODE_MEDIASELECT: SDL_Scancode = 263;
pub const SDL_SCANCODE_AUDIOMUTE: SDL_Scancode = 262;
pub const SDL_SCANCODE_AUDIOPLAY: SDL_Scancode = 261;
pub const SDL_SCANCODE_AUDIOSTOP: SDL_Scancode = 260;
pub const SDL_SCANCODE_AUDIOPREV: SDL_Scancode = 259;
pub const SDL_SCANCODE_AUDIONEXT: SDL_Scancode = 258;
pub const SDL_SCANCODE_MODE: SDL_Scancode = 257;
pub const SDL_SCANCODE_RGUI: SDL_Scancode = 231;
pub const SDL_SCANCODE_RALT: SDL_Scancode = 230;
pub const SDL_SCANCODE_RSHIFT: SDL_Scancode = 229;
pub const SDL_SCANCODE_RCTRL: SDL_Scancode = 228;
pub const SDL_SCANCODE_LGUI: SDL_Scancode = 227;
pub const SDL_SCANCODE_LALT: SDL_Scancode = 226;
pub const SDL_SCANCODE_LSHIFT: SDL_Scancode = 225;
pub const SDL_SCANCODE_LCTRL: SDL_Scancode = 224;
pub const SDL_SCANCODE_KP_HEXADECIMAL: SDL_Scancode = 221;
pub const SDL_SCANCODE_KP_DECIMAL: SDL_Scancode = 220;
pub const SDL_SCANCODE_KP_OCTAL: SDL_Scancode = 219;
pub const SDL_SCANCODE_KP_BINARY: SDL_Scancode = 218;
pub const SDL_SCANCODE_KP_CLEARENTRY: SDL_Scancode = 217;
pub const SDL_SCANCODE_KP_CLEAR: SDL_Scancode = 216;
pub const SDL_SCANCODE_KP_PLUSMINUS: SDL_Scancode = 215;
pub const SDL_SCANCODE_KP_MEMDIVIDE: SDL_Scancode = 214;
pub const SDL_SCANCODE_KP_MEMMULTIPLY: SDL_Scancode = 213;
pub const SDL_SCANCODE_KP_MEMSUBTRACT: SDL_Scancode = 212;
pub const SDL_SCANCODE_KP_MEMADD: SDL_Scancode = 211;
pub const SDL_SCANCODE_KP_MEMCLEAR: SDL_Scancode = 210;
pub const SDL_SCANCODE_KP_MEMRECALL: SDL_Scancode = 209;
pub const SDL_SCANCODE_KP_MEMSTORE: SDL_Scancode = 208;
pub const SDL_SCANCODE_KP_EXCLAM: SDL_Scancode = 207;
pub const SDL_SCANCODE_KP_AT: SDL_Scancode = 206;
pub const SDL_SCANCODE_KP_SPACE: SDL_Scancode = 205;
pub const SDL_SCANCODE_KP_HASH: SDL_Scancode = 204;
pub const SDL_SCANCODE_KP_COLON: SDL_Scancode = 203;
pub const SDL_SCANCODE_KP_DBLVERTICALBAR: SDL_Scancode = 202;
pub const SDL_SCANCODE_KP_VERTICALBAR: SDL_Scancode = 201;
pub const SDL_SCANCODE_KP_DBLAMPERSAND: SDL_Scancode = 200;
pub const SDL_SCANCODE_KP_AMPERSAND: SDL_Scancode = 199;
pub const SDL_SCANCODE_KP_GREATER: SDL_Scancode = 198;
pub const SDL_SCANCODE_KP_LESS: SDL_Scancode = 197;
pub const SDL_SCANCODE_KP_PERCENT: SDL_Scancode = 196;
pub const SDL_SCANCODE_KP_POWER: SDL_Scancode = 195;
pub const SDL_SCANCODE_KP_XOR: SDL_Scancode = 194;
pub const SDL_SCANCODE_KP_F: SDL_Scancode = 193;
pub const SDL_SCANCODE_KP_E: SDL_Scancode = 192;
pub const SDL_SCANCODE_KP_D: SDL_Scancode = 191;
pub const SDL_SCANCODE_KP_C: SDL_Scancode = 190;
pub const SDL_SCANCODE_KP_B: SDL_Scancode = 189;
pub const SDL_SCANCODE_KP_A: SDL_Scancode = 188;
pub const SDL_SCANCODE_KP_BACKSPACE: SDL_Scancode = 187;
pub const SDL_SCANCODE_KP_TAB: SDL_Scancode = 186;
pub const SDL_SCANCODE_KP_RIGHTBRACE: SDL_Scancode = 185;
pub const SDL_SCANCODE_KP_LEFTBRACE: SDL_Scancode = 184;
pub const SDL_SCANCODE_KP_RIGHTPAREN: SDL_Scancode = 183;
pub const SDL_SCANCODE_KP_LEFTPAREN: SDL_Scancode = 182;
pub const SDL_SCANCODE_CURRENCYSUBUNIT: SDL_Scancode = 181;
pub const SDL_SCANCODE_CURRENCYUNIT: SDL_Scancode = 180;
pub const SDL_SCANCODE_DECIMALSEPARATOR: SDL_Scancode = 179;
pub const SDL_SCANCODE_THOUSANDSSEPARATOR: SDL_Scancode = 178;
pub const SDL_SCANCODE_KP_000: SDL_Scancode = 177;
pub const SDL_SCANCODE_KP_00: SDL_Scancode = 176;
pub const SDL_SCANCODE_EXSEL: SDL_Scancode = 164;
pub const SDL_SCANCODE_CRSEL: SDL_Scancode = 163;
pub const SDL_SCANCODE_CLEARAGAIN: SDL_Scancode = 162;
pub const SDL_SCANCODE_OPER: SDL_Scancode = 161;
pub const SDL_SCANCODE_OUT: SDL_Scancode = 160;
pub const SDL_SCANCODE_SEPARATOR: SDL_Scancode = 159;
pub const SDL_SCANCODE_RETURN2: SDL_Scancode = 158;
pub const SDL_SCANCODE_PRIOR: SDL_Scancode = 157;
pub const SDL_SCANCODE_CLEAR: SDL_Scancode = 156;
pub const SDL_SCANCODE_CANCEL: SDL_Scancode = 155;
pub const SDL_SCANCODE_SYSREQ: SDL_Scancode = 154;
pub const SDL_SCANCODE_ALTERASE: SDL_Scancode = 153;
pub const SDL_SCANCODE_LANG9: SDL_Scancode = 152;
pub const SDL_SCANCODE_LANG8: SDL_Scancode = 151;
pub const SDL_SCANCODE_LANG7: SDL_Scancode = 150;
pub const SDL_SCANCODE_LANG6: SDL_Scancode = 149;
pub const SDL_SCANCODE_LANG5: SDL_Scancode = 148;
pub const SDL_SCANCODE_LANG4: SDL_Scancode = 147;
pub const SDL_SCANCODE_LANG3: SDL_Scancode = 146;
pub const SDL_SCANCODE_LANG2: SDL_Scancode = 145;
pub const SDL_SCANCODE_LANG1: SDL_Scancode = 144;
pub const SDL_SCANCODE_INTERNATIONAL9: SDL_Scancode = 143;
pub const SDL_SCANCODE_INTERNATIONAL8: SDL_Scancode = 142;
pub const SDL_SCANCODE_INTERNATIONAL7: SDL_Scancode = 141;
pub const SDL_SCANCODE_INTERNATIONAL6: SDL_Scancode = 140;
pub const SDL_SCANCODE_INTERNATIONAL5: SDL_Scancode = 139;
pub const SDL_SCANCODE_INTERNATIONAL4: SDL_Scancode = 138;
pub const SDL_SCANCODE_INTERNATIONAL3: SDL_Scancode = 137;
pub const SDL_SCANCODE_INTERNATIONAL2: SDL_Scancode = 136;
pub const SDL_SCANCODE_INTERNATIONAL1: SDL_Scancode = 135;
pub const SDL_SCANCODE_KP_EQUALSAS400: SDL_Scancode = 134;
pub const SDL_SCANCODE_KP_COMMA: SDL_Scancode = 133;
pub const SDL_SCANCODE_VOLUMEDOWN: SDL_Scancode = 129;
pub const SDL_SCANCODE_VOLUMEUP: SDL_Scancode = 128;
pub const SDL_SCANCODE_MUTE: SDL_Scancode = 127;
pub const SDL_SCANCODE_FIND: SDL_Scancode = 126;
pub const SDL_SCANCODE_PASTE: SDL_Scancode = 125;
pub const SDL_SCANCODE_COPY: SDL_Scancode = 124;
pub const SDL_SCANCODE_CUT: SDL_Scancode = 123;
pub const SDL_SCANCODE_UNDO: SDL_Scancode = 122;
pub const SDL_SCANCODE_AGAIN: SDL_Scancode = 121;
pub const SDL_SCANCODE_STOP: SDL_Scancode = 120;
pub const SDL_SCANCODE_SELECT: SDL_Scancode = 119;
pub const SDL_SCANCODE_MENU: SDL_Scancode = 118;
pub const SDL_SCANCODE_HELP: SDL_Scancode = 117;
pub const SDL_SCANCODE_EXECUTE: SDL_Scancode = 116;
pub const SDL_SCANCODE_F24: SDL_Scancode = 115;
pub const SDL_SCANCODE_F23: SDL_Scancode = 114;
pub const SDL_SCANCODE_F22: SDL_Scancode = 113;
pub const SDL_SCANCODE_F21: SDL_Scancode = 112;
pub const SDL_SCANCODE_F20: SDL_Scancode = 111;
pub const SDL_SCANCODE_F19: SDL_Scancode = 110;
pub const SDL_SCANCODE_F18: SDL_Scancode = 109;
pub const SDL_SCANCODE_F17: SDL_Scancode = 108;
pub const SDL_SCANCODE_F16: SDL_Scancode = 107;
pub const SDL_SCANCODE_F15: SDL_Scancode = 106;
pub const SDL_SCANCODE_F14: SDL_Scancode = 105;
pub const SDL_SCANCODE_F13: SDL_Scancode = 104;
pub const SDL_SCANCODE_KP_EQUALS: SDL_Scancode = 103;
pub const SDL_SCANCODE_POWER: SDL_Scancode = 102;
pub const SDL_SCANCODE_APPLICATION: SDL_Scancode = 101;
pub const SDL_SCANCODE_NONUSBACKSLASH: SDL_Scancode = 100;
pub const SDL_SCANCODE_KP_PERIOD: SDL_Scancode = 99;
pub const SDL_SCANCODE_KP_0: SDL_Scancode = 98;
pub const SDL_SCANCODE_KP_9: SDL_Scancode = 97;
pub const SDL_SCANCODE_KP_8: SDL_Scancode = 96;
pub const SDL_SCANCODE_KP_7: SDL_Scancode = 95;
pub const SDL_SCANCODE_KP_6: SDL_Scancode = 94;
pub const SDL_SCANCODE_KP_5: SDL_Scancode = 93;
pub const SDL_SCANCODE_KP_4: SDL_Scancode = 92;
pub const SDL_SCANCODE_KP_3: SDL_Scancode = 91;
pub const SDL_SCANCODE_KP_2: SDL_Scancode = 90;
pub const SDL_SCANCODE_KP_1: SDL_Scancode = 89;
pub const SDL_SCANCODE_KP_ENTER: SDL_Scancode = 88;
pub const SDL_SCANCODE_KP_PLUS: SDL_Scancode = 87;
pub const SDL_SCANCODE_KP_MINUS: SDL_Scancode = 86;
pub const SDL_SCANCODE_KP_MULTIPLY: SDL_Scancode = 85;
pub const SDL_SCANCODE_KP_DIVIDE: SDL_Scancode = 84;
pub const SDL_SCANCODE_NUMLOCKCLEAR: SDL_Scancode = 83;
pub const SDL_SCANCODE_UP: SDL_Scancode = 82;
pub const SDL_SCANCODE_DOWN: SDL_Scancode = 81;
pub const SDL_SCANCODE_LEFT: SDL_Scancode = 80;
pub const SDL_SCANCODE_RIGHT: SDL_Scancode = 79;
pub const SDL_SCANCODE_PAGEDOWN: SDL_Scancode = 78;
pub const SDL_SCANCODE_END: SDL_Scancode = 77;
pub const SDL_SCANCODE_DELETE: SDL_Scancode = 76;
pub const SDL_SCANCODE_PAGEUP: SDL_Scancode = 75;
pub const SDL_SCANCODE_HOME: SDL_Scancode = 74;
pub const SDL_SCANCODE_INSERT: SDL_Scancode = 73;
pub const SDL_SCANCODE_PAUSE: SDL_Scancode = 72;
pub const SDL_SCANCODE_SCROLLLOCK: SDL_Scancode = 71;
pub const SDL_SCANCODE_PRINTSCREEN: SDL_Scancode = 70;
pub const SDL_SCANCODE_F12: SDL_Scancode = 69;
pub const SDL_SCANCODE_F11: SDL_Scancode = 68;
pub const SDL_SCANCODE_F10: SDL_Scancode = 67;
pub const SDL_SCANCODE_F9: SDL_Scancode = 66;
pub const SDL_SCANCODE_F8: SDL_Scancode = 65;
pub const SDL_SCANCODE_F7: SDL_Scancode = 64;
pub const SDL_SCANCODE_F6: SDL_Scancode = 63;
pub const SDL_SCANCODE_F5: SDL_Scancode = 62;
pub const SDL_SCANCODE_F4: SDL_Scancode = 61;
pub const SDL_SCANCODE_F3: SDL_Scancode = 60;
pub const SDL_SCANCODE_F2: SDL_Scancode = 59;
pub const SDL_SCANCODE_F1: SDL_Scancode = 58;
pub const SDL_SCANCODE_CAPSLOCK: SDL_Scancode = 57;
pub const SDL_SCANCODE_SLASH: SDL_Scancode = 56;
pub const SDL_SCANCODE_PERIOD: SDL_Scancode = 55;
pub const SDL_SCANCODE_COMMA: SDL_Scancode = 54;
pub const SDL_SCANCODE_GRAVE: SDL_Scancode = 53;
pub const SDL_SCANCODE_APOSTROPHE: SDL_Scancode = 52;
pub const SDL_SCANCODE_SEMICOLON: SDL_Scancode = 51;
pub const SDL_SCANCODE_NONUSHASH: SDL_Scancode = 50;
pub const SDL_SCANCODE_BACKSLASH: SDL_Scancode = 49;
pub const SDL_SCANCODE_RIGHTBRACKET: SDL_Scancode = 48;
pub const SDL_SCANCODE_LEFTBRACKET: SDL_Scancode = 47;
pub const SDL_SCANCODE_EQUALS: SDL_Scancode = 46;
pub const SDL_SCANCODE_MINUS: SDL_Scancode = 45;
pub const SDL_SCANCODE_SPACE: SDL_Scancode = 44;
pub const SDL_SCANCODE_TAB: SDL_Scancode = 43;
pub const SDL_SCANCODE_BACKSPACE: SDL_Scancode = 42;
pub const SDL_SCANCODE_ESCAPE: SDL_Scancode = 41;
pub const SDL_SCANCODE_RETURN: SDL_Scancode = 40;
pub const SDL_SCANCODE_0: SDL_Scancode = 39;
pub const SDL_SCANCODE_9: SDL_Scancode = 38;
pub const SDL_SCANCODE_8: SDL_Scancode = 37;
pub const SDL_SCANCODE_7: SDL_Scancode = 36;
pub const SDL_SCANCODE_6: SDL_Scancode = 35;
pub const SDL_SCANCODE_5: SDL_Scancode = 34;
pub const SDL_SCANCODE_4: SDL_Scancode = 33;
pub const SDL_SCANCODE_3: SDL_Scancode = 32;
pub const SDL_SCANCODE_2: SDL_Scancode = 31;
pub const SDL_SCANCODE_1: SDL_Scancode = 30;
pub const SDL_SCANCODE_Z: SDL_Scancode = 29;
pub const SDL_SCANCODE_Y: SDL_Scancode = 28;
pub const SDL_SCANCODE_X: SDL_Scancode = 27;
pub const SDL_SCANCODE_W: SDL_Scancode = 26;
pub const SDL_SCANCODE_V: SDL_Scancode = 25;
pub const SDL_SCANCODE_U: SDL_Scancode = 24;
pub const SDL_SCANCODE_T: SDL_Scancode = 23;
pub const SDL_SCANCODE_S: SDL_Scancode = 22;
pub const SDL_SCANCODE_R: SDL_Scancode = 21;
pub const SDL_SCANCODE_Q: SDL_Scancode = 20;
pub const SDL_SCANCODE_P: SDL_Scancode = 19;
pub const SDL_SCANCODE_O: SDL_Scancode = 18;
pub const SDL_SCANCODE_N: SDL_Scancode = 17;
pub const SDL_SCANCODE_M: SDL_Scancode = 16;
pub const SDL_SCANCODE_L: SDL_Scancode = 15;
pub const SDL_SCANCODE_K: SDL_Scancode = 14;
pub const SDL_SCANCODE_J: SDL_Scancode = 13;
pub const SDL_SCANCODE_I: SDL_Scancode = 12;
pub const SDL_SCANCODE_H: SDL_Scancode = 11;
pub const SDL_SCANCODE_G: SDL_Scancode = 10;
pub const SDL_SCANCODE_F: SDL_Scancode = 9;
pub const SDL_SCANCODE_E: SDL_Scancode = 8;
pub const SDL_SCANCODE_D: SDL_Scancode = 7;
pub const SDL_SCANCODE_C: SDL_Scancode = 6;
pub const SDL_SCANCODE_B: SDL_Scancode = 5;
pub const SDL_SCANCODE_A: SDL_Scancode = 4;
pub const SDL_SCANCODE_UNKNOWN: SDL_Scancode = 0;
pub type SDL_GameControllerAxis = libc::c_int;
pub const SDL_CONTROLLER_AXIS_MAX: SDL_GameControllerAxis = 6;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: SDL_GameControllerAxis = 5;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: SDL_GameControllerAxis = 4;
pub const SDL_CONTROLLER_AXIS_RIGHTY: SDL_GameControllerAxis = 3;
pub const SDL_CONTROLLER_AXIS_RIGHTX: SDL_GameControllerAxis = 2;
pub const SDL_CONTROLLER_AXIS_LEFTY: SDL_GameControllerAxis = 1;
pub const SDL_CONTROLLER_AXIS_LEFTX: SDL_GameControllerAxis = 0;
pub const SDL_CONTROLLER_AXIS_INVALID: SDL_GameControllerAxis = -1;
pub type SDL_GameControllerButton = libc::c_int;
pub const SDL_CONTROLLER_BUTTON_MAX: SDL_GameControllerButton = 21;
pub const SDL_CONTROLLER_BUTTON_TOUCHPAD: SDL_GameControllerButton = 20;
pub const SDL_CONTROLLER_BUTTON_PADDLE4: SDL_GameControllerButton = 19;
pub const SDL_CONTROLLER_BUTTON_PADDLE3: SDL_GameControllerButton = 18;
pub const SDL_CONTROLLER_BUTTON_PADDLE2: SDL_GameControllerButton = 17;
pub const SDL_CONTROLLER_BUTTON_PADDLE1: SDL_GameControllerButton = 16;
pub const SDL_CONTROLLER_BUTTON_MISC1: SDL_GameControllerButton = 15;
pub const SDL_CONTROLLER_BUTTON_DPAD_RIGHT: SDL_GameControllerButton = 14;
pub const SDL_CONTROLLER_BUTTON_DPAD_LEFT: SDL_GameControllerButton = 13;
pub const SDL_CONTROLLER_BUTTON_DPAD_DOWN: SDL_GameControllerButton = 12;
pub const SDL_CONTROLLER_BUTTON_DPAD_UP: SDL_GameControllerButton = 11;
pub const SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: SDL_GameControllerButton = 10;
pub const SDL_CONTROLLER_BUTTON_LEFTSHOULDER: SDL_GameControllerButton = 9;
pub const SDL_CONTROLLER_BUTTON_RIGHTSTICK: SDL_GameControllerButton = 8;
pub const SDL_CONTROLLER_BUTTON_LEFTSTICK: SDL_GameControllerButton = 7;
pub const SDL_CONTROLLER_BUTTON_START: SDL_GameControllerButton = 6;
pub const SDL_CONTROLLER_BUTTON_GUIDE: SDL_GameControllerButton = 5;
pub const SDL_CONTROLLER_BUTTON_BACK: SDL_GameControllerButton = 4;
pub const SDL_CONTROLLER_BUTTON_Y: SDL_GameControllerButton = 3;
pub const SDL_CONTROLLER_BUTTON_X: SDL_GameControllerButton = 2;
pub const SDL_CONTROLLER_BUTTON_B: SDL_GameControllerButton = 1;
pub const SDL_CONTROLLER_BUTTON_A: SDL_GameControllerButton = 0;
pub const SDL_CONTROLLER_BUTTON_INVALID: SDL_GameControllerButton = -1;
#[no_mangle]
pub static mut DeviceType_Null: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Mouse: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Keyboard: DeviceType = 0;
#[no_mangle]
pub static mut DeviceType_Gamepad: DeviceType = 0;
#[no_mangle]
pub static mut Button_Null: Button = 0 as libc::c_int;
#[no_mangle]
pub static mut Button_First: Button = 1 as libc::c_int;
#[no_mangle]
pub static mut Button_Keyboard_First: Button = unsafe { Button_First };
#[no_mangle]
pub static mut Button_Keyboard_A: Button = unsafe {
    Button_Keyboard_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_B: Button = unsafe {
    Button_Keyboard_First + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_C: Button = unsafe {
    Button_Keyboard_First + 2 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_D: Button = unsafe {
    Button_Keyboard_First + 3 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_E: Button = unsafe {
    Button_Keyboard_First + 4 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F: Button = unsafe {
    Button_Keyboard_First + 5 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_G: Button = unsafe {
    Button_Keyboard_First + 6 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_H: Button = unsafe {
    Button_Keyboard_First + 7 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_I: Button = unsafe {
    Button_Keyboard_First + 8 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_J: Button = unsafe {
    Button_Keyboard_First + 9 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_K: Button = unsafe {
    Button_Keyboard_First + 10 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_L: Button = unsafe {
    Button_Keyboard_First + 11 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_M: Button = unsafe {
    Button_Keyboard_First + 12 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N: Button = unsafe {
    Button_Keyboard_First + 13 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_O: Button = unsafe {
    Button_Keyboard_First + 14 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_P: Button = unsafe {
    Button_Keyboard_First + 15 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Q: Button = unsafe {
    Button_Keyboard_First + 16 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_R: Button = unsafe {
    Button_Keyboard_First + 17 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_S: Button = unsafe {
    Button_Keyboard_First + 18 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_T: Button = unsafe {
    Button_Keyboard_First + 19 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_U: Button = unsafe {
    Button_Keyboard_First + 20 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_V: Button = unsafe {
    Button_Keyboard_First + 21 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_W: Button = unsafe {
    Button_Keyboard_First + 22 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_X: Button = unsafe {
    Button_Keyboard_First + 23 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Y: Button = unsafe {
    Button_Keyboard_First + 24 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Z: Button = unsafe {
    Button_Keyboard_First + 25 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N0: Button = unsafe {
    Button_Keyboard_First + 26 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N1: Button = unsafe {
    Button_Keyboard_First + 27 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N2: Button = unsafe {
    Button_Keyboard_First + 28 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N3: Button = unsafe {
    Button_Keyboard_First + 29 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N4: Button = unsafe {
    Button_Keyboard_First + 30 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N5: Button = unsafe {
    Button_Keyboard_First + 31 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N6: Button = unsafe {
    Button_Keyboard_First + 32 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N7: Button = unsafe {
    Button_Keyboard_First + 33 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N8: Button = unsafe {
    Button_Keyboard_First + 34 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_N9: Button = unsafe {
    Button_Keyboard_First + 35 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F1: Button = unsafe {
    Button_Keyboard_First + 36 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F2: Button = unsafe {
    Button_Keyboard_First + 37 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F3: Button = unsafe {
    Button_Keyboard_First + 38 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F4: Button = unsafe {
    Button_Keyboard_First + 39 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F5: Button = unsafe {
    Button_Keyboard_First + 40 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F6: Button = unsafe {
    Button_Keyboard_First + 41 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F7: Button = unsafe {
    Button_Keyboard_First + 42 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F8: Button = unsafe {
    Button_Keyboard_First + 43 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F9: Button = unsafe {
    Button_Keyboard_First + 44 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F10: Button = unsafe {
    Button_Keyboard_First + 45 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F11: Button = unsafe {
    Button_Keyboard_First + 46 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F12: Button = unsafe {
    Button_Keyboard_First + 47 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F13: Button = unsafe {
    Button_Keyboard_First + 48 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F14: Button = unsafe {
    Button_Keyboard_First + 49 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F15: Button = unsafe {
    Button_Keyboard_First + 50 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F16: Button = unsafe {
    Button_Keyboard_First + 51 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F17: Button = unsafe {
    Button_Keyboard_First + 52 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F18: Button = unsafe {
    Button_Keyboard_First + 53 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F19: Button = unsafe {
    Button_Keyboard_First + 54 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F20: Button = unsafe {
    Button_Keyboard_First + 55 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F21: Button = unsafe {
    Button_Keyboard_First + 56 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F22: Button = unsafe {
    Button_Keyboard_First + 57 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F23: Button = unsafe {
    Button_Keyboard_First + 58 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_F24: Button = unsafe {
    Button_Keyboard_First + 59 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP0: Button = unsafe {
    Button_Keyboard_First + 60 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP1: Button = unsafe {
    Button_Keyboard_First + 61 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP2: Button = unsafe {
    Button_Keyboard_First + 62 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP3: Button = unsafe {
    Button_Keyboard_First + 63 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP4: Button = unsafe {
    Button_Keyboard_First + 64 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP5: Button = unsafe {
    Button_Keyboard_First + 65 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP6: Button = unsafe {
    Button_Keyboard_First + 66 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP7: Button = unsafe {
    Button_Keyboard_First + 67 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP8: Button = unsafe {
    Button_Keyboard_First + 68 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KP9: Button = unsafe {
    Button_Keyboard_First + 69 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPNumLock: Button = unsafe {
    Button_Keyboard_First + 70 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPDivide: Button = unsafe {
    Button_Keyboard_First + 71 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPMultiply: Button = unsafe {
    Button_Keyboard_First + 72 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPSubtract: Button = unsafe {
    Button_Keyboard_First + 73 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPAdd: Button = unsafe {
    Button_Keyboard_First + 74 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPEnter: Button = unsafe {
    Button_Keyboard_First + 75 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_KPDecimal: Button = unsafe {
    Button_Keyboard_First + 76 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Backspace: Button = unsafe {
    Button_Keyboard_First + 77 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Escape: Button = unsafe {
    Button_Keyboard_First + 78 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Return: Button = unsafe {
    Button_Keyboard_First + 79 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Space: Button = unsafe {
    Button_Keyboard_First + 80 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Tab: Button = unsafe {
    Button_Keyboard_First + 81 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Backtick: Button = unsafe {
    Button_Keyboard_First + 82 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_CapsLock: Button = unsafe {
    Button_Keyboard_First + 83 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Minus: Button = unsafe {
    Button_Keyboard_First + 84 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Equals: Button = unsafe {
    Button_Keyboard_First + 85 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_LBracket: Button = unsafe {
    Button_Keyboard_First + 86 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_RBracket: Button = unsafe {
    Button_Keyboard_First + 87 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Backslash: Button = unsafe {
    Button_Keyboard_First + 88 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Semicolon: Button = unsafe {
    Button_Keyboard_First + 89 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Apostrophe: Button = unsafe {
    Button_Keyboard_First + 90 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Comma: Button = unsafe {
    Button_Keyboard_First + 91 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Period: Button = unsafe {
    Button_Keyboard_First + 92 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Slash: Button = unsafe {
    Button_Keyboard_First + 93 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_PrintScreen: Button = unsafe {
    Button_Keyboard_First + 94 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_ScrollLock: Button = unsafe {
    Button_Keyboard_First + 95 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Pause: Button = unsafe {
    Button_Keyboard_First + 96 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Insert: Button = unsafe {
    Button_Keyboard_First + 97 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Delete: Button = unsafe {
    Button_Keyboard_First + 98 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Home: Button = unsafe {
    Button_Keyboard_First + 99 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_End: Button = unsafe {
    Button_Keyboard_First + 100 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_PageUp: Button = unsafe {
    Button_Keyboard_First + 101 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_PageDown: Button = unsafe {
    Button_Keyboard_First + 102 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Right: Button = unsafe {
    Button_Keyboard_First + 103 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Left: Button = unsafe {
    Button_Keyboard_First + 104 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Down: Button = unsafe {
    Button_Keyboard_First + 105 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Up: Button = unsafe {
    Button_Keyboard_First + 106 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_LCtrl: Button = unsafe {
    Button_Keyboard_First + 107 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_LShift: Button = unsafe {
    Button_Keyboard_First + 108 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_LAlt: Button = unsafe {
    Button_Keyboard_First + 109 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_LMeta: Button = unsafe {
    Button_Keyboard_First + 110 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_RCtrl: Button = unsafe {
    Button_Keyboard_First + 111 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_RShift: Button = unsafe {
    Button_Keyboard_First + 112 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_RAlt: Button = unsafe {
    Button_Keyboard_First + 113 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_RMeta: Button = unsafe {
    Button_Keyboard_First + 114 as libc::c_int
};
#[no_mangle]
pub static mut Button_Keyboard_Last: Button = unsafe {
    Button_Keyboard_First + 115 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_First: Button = unsafe {
    Button_Keyboard_Last + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_Left: Button = unsafe {
    Button_Mouse_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_Middle: Button = unsafe {
    Button_Mouse_First + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_Right: Button = unsafe {
    Button_Mouse_First + 2 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_X1: Button = unsafe {
    Button_Mouse_First + 3 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_X2: Button = unsafe {
    Button_Mouse_First + 4 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_X: Button = unsafe { Button_Mouse_First + 5 as libc::c_int };
#[no_mangle]
pub static mut Button_Mouse_Y: Button = unsafe { Button_Mouse_First + 6 as libc::c_int };
#[no_mangle]
pub static mut Button_Mouse_ScrollX: Button = unsafe {
    Button_Mouse_First + 7 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_ScrollY: Button = unsafe {
    Button_Mouse_First + 8 as libc::c_int
};
#[no_mangle]
pub static mut Button_Mouse_Last: Button = unsafe {
    Button_Mouse_First + 8 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_First: Button = unsafe {
    Button_Mouse_Last + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Button_First: Button = unsafe {
    Button_Gamepad_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_A: Button = unsafe {
    Button_Gamepad_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_B: Button = unsafe {
    Button_Gamepad_First + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_X: Button = unsafe {
    Button_Gamepad_First + 2 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Y: Button = unsafe {
    Button_Gamepad_First + 3 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Back: Button = unsafe {
    Button_Gamepad_First + 4 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Guide: Button = unsafe {
    Button_Gamepad_First + 5 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Start: Button = unsafe {
    Button_Gamepad_First + 6 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_LStick: Button = unsafe {
    Button_Gamepad_First + 7 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_RStick: Button = unsafe {
    Button_Gamepad_First + 8 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_LBumper: Button = unsafe {
    Button_Gamepad_First + 9 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_RBumper: Button = unsafe {
    Button_Gamepad_First + 10 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Up: Button = unsafe {
    Button_Gamepad_First + 11 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Down: Button = unsafe {
    Button_Gamepad_First + 12 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Left: Button = unsafe {
    Button_Gamepad_First + 13 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Right: Button = unsafe {
    Button_Gamepad_First + 14 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Button_Last: Button = unsafe {
    Button_Gamepad_First + 14 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Axis_First: Button = unsafe {
    Button_Gamepad_First + 15 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_LTrigger: Button = unsafe {
    Button_Gamepad_First + 15 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_RTrigger: Button = unsafe {
    Button_Gamepad_First + 16 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_LStickX: Button = unsafe {
    Button_Gamepad_First + 17 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_LStickY: Button = unsafe {
    Button_Gamepad_First + 18 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_RStickX: Button = unsafe {
    Button_Gamepad_First + 19 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_RStickY: Button = unsafe {
    Button_Gamepad_First + 20 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Axis_Last: Button = unsafe {
    Button_Gamepad_First + 20 as libc::c_int
};
#[no_mangle]
pub static mut Button_Gamepad_Last: Button = unsafe {
    Button_Gamepad_First + 20 as libc::c_int
};
#[no_mangle]
pub static mut Button_System_First: Button = unsafe {
    Button_Gamepad_Last + 1 as libc::c_int
};
#[no_mangle]
pub static mut Button_System_Exit: Button = unsafe {
    Button_System_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_System_Last: Button = unsafe {
    Button_System_First + 0 as libc::c_int
};
#[no_mangle]
pub static mut Button_Last: Button = unsafe { Button_System_Last };
#[no_mangle]
pub unsafe extern "C" fn Button_ToDeviceType(mut button: Button) -> DeviceType {
    if button == Button_Null {
        return DeviceType_Null
    } else if button <= Button_Keyboard_Last {
        return DeviceType_Keyboard
    } else if button <= Button_Mouse_Last {
        return DeviceType_Mouse
    } else if button <= Button_Gamepad_Last {
        return DeviceType_Gamepad
    } else if button <= Button_System_Last {
        return DeviceType_Null
    } else {
        Fatal(
            b"Button_ToDeviceType: Unknown Button: %i\0" as *const u8
                as *const libc::c_char,
            button,
        );
        return DeviceType_Null;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToString(mut button: Button) -> cstr {
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
            snprintf(
                buffer.as_mut_ptr(),
                (::core::mem::size_of::<[libc::c_char; 512]>())
                    .wrapping_div(
                        ::core::mem::size_of::<libc::c_char>() as usize,
                    ) as libc::c_int as libc::size_t,
                b"Unknown (%i)\0" as *const u8 as *const libc::c_char,
                button,
            );
            return buffer.as_mut_ptr() as cstr;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_IsAutoRelease(mut button: Button) -> bool {
    match button {
        124 | 125 | 147 => return 1 as libc::c_int != 0,
        _ => return 0 as libc::c_int != 0,
    };
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLScancode(mut scancode: SDL_Scancode) -> Button {
    match scancode as libc::c_uint {
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
        1 => return SDL_SCANCODE_A,
        2 => return SDL_SCANCODE_B,
        3 => return SDL_SCANCODE_C,
        4 => return SDL_SCANCODE_D,
        5 => return SDL_SCANCODE_E,
        6 => return SDL_SCANCODE_F,
        7 => return SDL_SCANCODE_G,
        8 => return SDL_SCANCODE_H,
        9 => return SDL_SCANCODE_I,
        10 => return SDL_SCANCODE_J,
        11 => return SDL_SCANCODE_K,
        12 => return SDL_SCANCODE_L,
        13 => return SDL_SCANCODE_M,
        14 => return SDL_SCANCODE_N,
        15 => return SDL_SCANCODE_O,
        16 => return SDL_SCANCODE_P,
        17 => return SDL_SCANCODE_Q,
        18 => return SDL_SCANCODE_R,
        19 => return SDL_SCANCODE_S,
        20 => return SDL_SCANCODE_T,
        21 => return SDL_SCANCODE_U,
        22 => return SDL_SCANCODE_V,
        23 => return SDL_SCANCODE_W,
        24 => return SDL_SCANCODE_X,
        25 => return SDL_SCANCODE_Y,
        26 => return SDL_SCANCODE_Z,
        27 => return SDL_SCANCODE_0,
        28 => return SDL_SCANCODE_1,
        29 => return SDL_SCANCODE_2,
        30 => return SDL_SCANCODE_3,
        31 => return SDL_SCANCODE_4,
        32 => return SDL_SCANCODE_5,
        33 => return SDL_SCANCODE_6,
        34 => return SDL_SCANCODE_7,
        35 => return SDL_SCANCODE_8,
        36 => return SDL_SCANCODE_9,
        37 => return SDL_SCANCODE_F1,
        38 => return SDL_SCANCODE_F2,
        39 => return SDL_SCANCODE_F3,
        40 => return SDL_SCANCODE_F4,
        41 => return SDL_SCANCODE_F5,
        42 => return SDL_SCANCODE_F6,
        43 => return SDL_SCANCODE_F7,
        44 => return SDL_SCANCODE_F8,
        45 => return SDL_SCANCODE_F9,
        46 => return SDL_SCANCODE_F10,
        47 => return SDL_SCANCODE_F11,
        48 => return SDL_SCANCODE_F12,
        49 => return SDL_SCANCODE_F13,
        50 => return SDL_SCANCODE_F14,
        51 => return SDL_SCANCODE_F15,
        52 => return SDL_SCANCODE_F16,
        53 => return SDL_SCANCODE_F17,
        54 => return SDL_SCANCODE_F18,
        55 => return SDL_SCANCODE_F19,
        56 => return SDL_SCANCODE_F20,
        57 => return SDL_SCANCODE_F21,
        58 => return SDL_SCANCODE_F22,
        59 => return SDL_SCANCODE_F23,
        60 => return SDL_SCANCODE_F24,
        61 => return SDL_SCANCODE_KP_0,
        62 => return SDL_SCANCODE_KP_1,
        63 => return SDL_SCANCODE_KP_2,
        64 => return SDL_SCANCODE_KP_3,
        65 => return SDL_SCANCODE_KP_4,
        66 => return SDL_SCANCODE_KP_5,
        67 => return SDL_SCANCODE_KP_6,
        68 => return SDL_SCANCODE_KP_7,
        69 => return SDL_SCANCODE_KP_8,
        70 => return SDL_SCANCODE_KP_9,
        71 => return SDL_SCANCODE_NUMLOCKCLEAR,
        72 => return SDL_SCANCODE_KP_DIVIDE,
        73 => return SDL_SCANCODE_KP_MULTIPLY,
        74 => return SDL_SCANCODE_KP_MINUS,
        75 => return SDL_SCANCODE_KP_PLUS,
        76 => return SDL_SCANCODE_KP_ENTER,
        77 => return SDL_SCANCODE_KP_DECIMAL,
        78 => return SDL_SCANCODE_BACKSPACE,
        79 => return SDL_SCANCODE_ESCAPE,
        80 => return SDL_SCANCODE_RETURN,
        81 => return SDL_SCANCODE_SPACE,
        82 => return SDL_SCANCODE_TAB,
        83 => return SDL_SCANCODE_GRAVE,
        84 => return SDL_SCANCODE_CAPSLOCK,
        85 => return SDL_SCANCODE_MINUS,
        86 => return SDL_SCANCODE_EQUALS,
        87 => return SDL_SCANCODE_LEFTBRACKET,
        88 => return SDL_SCANCODE_RIGHTBRACKET,
        89 => return SDL_SCANCODE_BACKSLASH,
        90 => return SDL_SCANCODE_SEMICOLON,
        91 => return SDL_SCANCODE_APOSTROPHE,
        92 => return SDL_SCANCODE_COMMA,
        93 => return SDL_SCANCODE_PERIOD,
        94 => return SDL_SCANCODE_SLASH,
        95 => return SDL_SCANCODE_PRINTSCREEN,
        96 => return SDL_SCANCODE_SCROLLLOCK,
        97 => return SDL_SCANCODE_PAUSE,
        98 => return SDL_SCANCODE_INSERT,
        99 => return SDL_SCANCODE_DELETE,
        100 => return SDL_SCANCODE_HOME,
        101 => return SDL_SCANCODE_END,
        102 => return SDL_SCANCODE_PAGEUP,
        103 => return SDL_SCANCODE_PAGEDOWN,
        104 => return SDL_SCANCODE_RIGHT,
        105 => return SDL_SCANCODE_LEFT,
        106 => return SDL_SCANCODE_DOWN,
        107 => return SDL_SCANCODE_UP,
        108 => return SDL_SCANCODE_LCTRL,
        109 => return SDL_SCANCODE_LSHIFT,
        110 => return SDL_SCANCODE_LALT,
        111 => return SDL_SCANCODE_LGUI,
        112 => return SDL_SCANCODE_RCTRL,
        113 => return SDL_SCANCODE_RSHIFT,
        114 => return SDL_SCANCODE_RALT,
        115 => return SDL_SCANCODE_RGUI,
        _ => {
            Fatal(
                b"Button_ToSDLScancode: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_SCANCODE_UNKNOWN;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLMouseButton(mut mouseButton: u8) -> Button {
    match mouseButton as libc::c_int {
        1 => {}
        2 => return Button_Mouse_Middle,
        3 => return Button_Mouse_Right,
        4 => return Button_Mouse_X1,
        5 => return Button_Mouse_X2,
        _ => {
            Fatal(
                b"Button_FromSDLMouseButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                mouseButton as libc::c_int,
            );
        }
    }
    return Button_Mouse_Left;
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLMouseButton(mut button: Button) -> u8 {
    match button {
        117 => {}
        118 => return 2 as libc::c_int as u8,
        119 => return 3 as libc::c_int as u8,
        120 => return 4 as libc::c_int as u8,
        121 => return 5 as libc::c_int as u8,
        _ => {
            Fatal(
                b"Button_ToSDLMouseButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return 1 as libc::c_int as u8;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerAxis(
    mut controllerAxis: SDL_GameControllerAxis,
) -> Button {
    match controllerAxis as libc::c_int {
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
                controllerAxis as libc::c_int,
            );
        }
    }
    return Button_Gamepad_LStickX;
}
#[no_mangle]
pub unsafe extern "C" fn Button_ToSDLControllerAxis(
    mut button: Button,
) -> SDL_GameControllerAxis {
    match button {
        143 => {}
        144 => return SDL_CONTROLLER_AXIS_LEFTY,
        145 => return SDL_CONTROLLER_AXIS_RIGHTX,
        146 => return SDL_CONTROLLER_AXIS_RIGHTY,
        141 => return SDL_CONTROLLER_AXIS_TRIGGERLEFT,
        142 => return SDL_CONTROLLER_AXIS_TRIGGERRIGHT,
        _ => {
            Fatal(
                b"Button_ToSDLControllerAxis: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_CONTROLLER_AXIS_LEFTX;
}
#[no_mangle]
pub unsafe extern "C" fn Button_FromSDLControllerButton(
    mut controllerButton: SDL_GameControllerButton,
) -> Button {
    match controllerButton as libc::c_int {
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
                controllerButton as libc::c_int,
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
        127 => return SDL_CONTROLLER_BUTTON_B,
        128 => return SDL_CONTROLLER_BUTTON_X,
        129 => return SDL_CONTROLLER_BUTTON_Y,
        130 => return SDL_CONTROLLER_BUTTON_BACK,
        131 => return SDL_CONTROLLER_BUTTON_GUIDE,
        132 => return SDL_CONTROLLER_BUTTON_START,
        133 => return SDL_CONTROLLER_BUTTON_LEFTSTICK,
        134 => return SDL_CONTROLLER_BUTTON_RIGHTSTICK,
        135 => return SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
        136 => return SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
        137 => return SDL_CONTROLLER_BUTTON_DPAD_UP,
        138 => return SDL_CONTROLLER_BUTTON_DPAD_DOWN,
        139 => return SDL_CONTROLLER_BUTTON_DPAD_LEFT,
        140 => return SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
        _ => {
            Fatal(
                b"Button_ToSDLControllerButton: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                button,
            );
        }
    }
    return SDL_CONTROLLER_BUTTON_A;
}
