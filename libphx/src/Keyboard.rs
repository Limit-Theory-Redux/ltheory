use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {
    fn SDL_GetPerformanceFrequency() -> u64;
    fn SDL_GetPerformanceCounter() -> u64;
    fn SDL_GetKeyboardState(numkeys: *mut i32) -> *const u8;
}

pub type uchar = libc::c_uchar;
pub type Key = uchar;
pub const SDL_SCANCODE_RALT: C2RustUnnamed = 230;
pub const SDL_SCANCODE_LALT: C2RustUnnamed = 226;
pub const SDL_SCANCODE_RCTRL: C2RustUnnamed = 228;
pub const SDL_SCANCODE_LCTRL: C2RustUnnamed = 224;
pub const SDL_SCANCODE_RSHIFT: C2RustUnnamed = 229;
pub const SDL_SCANCODE_LSHIFT: C2RustUnnamed = 225;
pub type C2RustUnnamed = u32;
pub const SDL_NUM_SCANCODES: C2RustUnnamed = 512;
pub const SDL_SCANCODE_ENDCALL: C2RustUnnamed = 290;
pub const SDL_SCANCODE_CALL: C2RustUnnamed = 289;
pub const SDL_SCANCODE_SOFTRIGHT: C2RustUnnamed = 288;
pub const SDL_SCANCODE_SOFTLEFT: C2RustUnnamed = 287;
pub const SDL_SCANCODE_AUDIOFASTFORWARD: C2RustUnnamed = 286;
pub const SDL_SCANCODE_AUDIOREWIND: C2RustUnnamed = 285;
pub const SDL_SCANCODE_APP2: C2RustUnnamed = 284;
pub const SDL_SCANCODE_APP1: C2RustUnnamed = 283;
pub const SDL_SCANCODE_SLEEP: C2RustUnnamed = 282;
pub const SDL_SCANCODE_EJECT: C2RustUnnamed = 281;
pub const SDL_SCANCODE_KBDILLUMUP: C2RustUnnamed = 280;
pub const SDL_SCANCODE_KBDILLUMDOWN: C2RustUnnamed = 279;
pub const SDL_SCANCODE_KBDILLUMTOGGLE: C2RustUnnamed = 278;
pub const SDL_SCANCODE_DISPLAYSWITCH: C2RustUnnamed = 277;
pub const SDL_SCANCODE_BRIGHTNESSUP: C2RustUnnamed = 276;
pub const SDL_SCANCODE_BRIGHTNESSDOWN: C2RustUnnamed = 275;
pub const SDL_SCANCODE_AC_BOOKMARKS: C2RustUnnamed = 274;
pub const SDL_SCANCODE_AC_REFRESH: C2RustUnnamed = 273;
pub const SDL_SCANCODE_AC_STOP: C2RustUnnamed = 272;
pub const SDL_SCANCODE_AC_FORWARD: C2RustUnnamed = 271;
pub const SDL_SCANCODE_AC_BACK: C2RustUnnamed = 270;
pub const SDL_SCANCODE_AC_HOME: C2RustUnnamed = 269;
pub const SDL_SCANCODE_AC_SEARCH: C2RustUnnamed = 268;
pub const SDL_SCANCODE_COMPUTER: C2RustUnnamed = 267;
pub const SDL_SCANCODE_CALCULATOR: C2RustUnnamed = 266;
pub const SDL_SCANCODE_MAIL: C2RustUnnamed = 265;
pub const SDL_SCANCODE_WWW: C2RustUnnamed = 264;
pub const SDL_SCANCODE_MEDIASELECT: C2RustUnnamed = 263;
pub const SDL_SCANCODE_AUDIOMUTE: C2RustUnnamed = 262;
pub const SDL_SCANCODE_AUDIOPLAY: C2RustUnnamed = 261;
pub const SDL_SCANCODE_AUDIOSTOP: C2RustUnnamed = 260;
pub const SDL_SCANCODE_AUDIOPREV: C2RustUnnamed = 259;
pub const SDL_SCANCODE_AUDIONEXT: C2RustUnnamed = 258;
pub const SDL_SCANCODE_MODE: C2RustUnnamed = 257;
pub const SDL_SCANCODE_RGUI: C2RustUnnamed = 231;
pub const SDL_SCANCODE_LGUI: C2RustUnnamed = 227;
pub const SDL_SCANCODE_KP_HEXADECIMAL: C2RustUnnamed = 221;
pub const SDL_SCANCODE_KP_DECIMAL: C2RustUnnamed = 220;
pub const SDL_SCANCODE_KP_OCTAL: C2RustUnnamed = 219;
pub const SDL_SCANCODE_KP_BINARY: C2RustUnnamed = 218;
pub const SDL_SCANCODE_KP_CLEARENTRY: C2RustUnnamed = 217;
pub const SDL_SCANCODE_KP_CLEAR: C2RustUnnamed = 216;
pub const SDL_SCANCODE_KP_PLUSMINUS: C2RustUnnamed = 215;
pub const SDL_SCANCODE_KP_MEMDIVIDE: C2RustUnnamed = 214;
pub const SDL_SCANCODE_KP_MEMMULTIPLY: C2RustUnnamed = 213;
pub const SDL_SCANCODE_KP_MEMSUBTRACT: C2RustUnnamed = 212;
pub const SDL_SCANCODE_KP_MEMADD: C2RustUnnamed = 211;
pub const SDL_SCANCODE_KP_MEMCLEAR: C2RustUnnamed = 210;
pub const SDL_SCANCODE_KP_MEMRECALL: C2RustUnnamed = 209;
pub const SDL_SCANCODE_KP_MEMSTORE: C2RustUnnamed = 208;
pub const SDL_SCANCODE_KP_EXCLAM: C2RustUnnamed = 207;
pub const SDL_SCANCODE_KP_AT: C2RustUnnamed = 206;
pub const SDL_SCANCODE_KP_SPACE: C2RustUnnamed = 205;
pub const SDL_SCANCODE_KP_HASH: C2RustUnnamed = 204;
pub const SDL_SCANCODE_KP_COLON: C2RustUnnamed = 203;
pub const SDL_SCANCODE_KP_DBLVERTICALBAR: C2RustUnnamed = 202;
pub const SDL_SCANCODE_KP_VERTICALBAR: C2RustUnnamed = 201;
pub const SDL_SCANCODE_KP_DBLAMPERSAND: C2RustUnnamed = 200;
pub const SDL_SCANCODE_KP_AMPERSAND: C2RustUnnamed = 199;
pub const SDL_SCANCODE_KP_GREATER: C2RustUnnamed = 198;
pub const SDL_SCANCODE_KP_LESS: C2RustUnnamed = 197;
pub const SDL_SCANCODE_KP_PERCENT: C2RustUnnamed = 196;
pub const SDL_SCANCODE_KP_POWER: C2RustUnnamed = 195;
pub const SDL_SCANCODE_KP_XOR: C2RustUnnamed = 194;
pub const SDL_SCANCODE_KP_F: C2RustUnnamed = 193;
pub const SDL_SCANCODE_KP_E: C2RustUnnamed = 192;
pub const SDL_SCANCODE_KP_D: C2RustUnnamed = 191;
pub const SDL_SCANCODE_KP_C: C2RustUnnamed = 190;
pub const SDL_SCANCODE_KP_B: C2RustUnnamed = 189;
pub const SDL_SCANCODE_KP_A: C2RustUnnamed = 188;
pub const SDL_SCANCODE_KP_BACKSPACE: C2RustUnnamed = 187;
pub const SDL_SCANCODE_KP_TAB: C2RustUnnamed = 186;
pub const SDL_SCANCODE_KP_RIGHTBRACE: C2RustUnnamed = 185;
pub const SDL_SCANCODE_KP_LEFTBRACE: C2RustUnnamed = 184;
pub const SDL_SCANCODE_KP_RIGHTPAREN: C2RustUnnamed = 183;
pub const SDL_SCANCODE_KP_LEFTPAREN: C2RustUnnamed = 182;
pub const SDL_SCANCODE_CURRENCYSUBUNIT: C2RustUnnamed = 181;
pub const SDL_SCANCODE_CURRENCYUNIT: C2RustUnnamed = 180;
pub const SDL_SCANCODE_DECIMALSEPARATOR: C2RustUnnamed = 179;
pub const SDL_SCANCODE_THOUSANDSSEPARATOR: C2RustUnnamed = 178;
pub const SDL_SCANCODE_KP_000: C2RustUnnamed = 177;
pub const SDL_SCANCODE_KP_00: C2RustUnnamed = 176;
pub const SDL_SCANCODE_EXSEL: C2RustUnnamed = 164;
pub const SDL_SCANCODE_CRSEL: C2RustUnnamed = 163;
pub const SDL_SCANCODE_CLEARAGAIN: C2RustUnnamed = 162;
pub const SDL_SCANCODE_OPER: C2RustUnnamed = 161;
pub const SDL_SCANCODE_OUT: C2RustUnnamed = 160;
pub const SDL_SCANCODE_SEPARATOR: C2RustUnnamed = 159;
pub const SDL_SCANCODE_RETURN2: C2RustUnnamed = 158;
pub const SDL_SCANCODE_PRIOR: C2RustUnnamed = 157;
pub const SDL_SCANCODE_CLEAR: C2RustUnnamed = 156;
pub const SDL_SCANCODE_CANCEL: C2RustUnnamed = 155;
pub const SDL_SCANCODE_SYSREQ: C2RustUnnamed = 154;
pub const SDL_SCANCODE_ALTERASE: C2RustUnnamed = 153;
pub const SDL_SCANCODE_LANG9: C2RustUnnamed = 152;
pub const SDL_SCANCODE_LANG8: C2RustUnnamed = 151;
pub const SDL_SCANCODE_LANG7: C2RustUnnamed = 150;
pub const SDL_SCANCODE_LANG6: C2RustUnnamed = 149;
pub const SDL_SCANCODE_LANG5: C2RustUnnamed = 148;
pub const SDL_SCANCODE_LANG4: C2RustUnnamed = 147;
pub const SDL_SCANCODE_LANG3: C2RustUnnamed = 146;
pub const SDL_SCANCODE_LANG2: C2RustUnnamed = 145;
pub const SDL_SCANCODE_LANG1: C2RustUnnamed = 144;
pub const SDL_SCANCODE_INTERNATIONAL9: C2RustUnnamed = 143;
pub const SDL_SCANCODE_INTERNATIONAL8: C2RustUnnamed = 142;
pub const SDL_SCANCODE_INTERNATIONAL7: C2RustUnnamed = 141;
pub const SDL_SCANCODE_INTERNATIONAL6: C2RustUnnamed = 140;
pub const SDL_SCANCODE_INTERNATIONAL5: C2RustUnnamed = 139;
pub const SDL_SCANCODE_INTERNATIONAL4: C2RustUnnamed = 138;
pub const SDL_SCANCODE_INTERNATIONAL3: C2RustUnnamed = 137;
pub const SDL_SCANCODE_INTERNATIONAL2: C2RustUnnamed = 136;
pub const SDL_SCANCODE_INTERNATIONAL1: C2RustUnnamed = 135;
pub const SDL_SCANCODE_KP_EQUALSAS400: C2RustUnnamed = 134;
pub const SDL_SCANCODE_KP_COMMA: C2RustUnnamed = 133;
pub const SDL_SCANCODE_VOLUMEDOWN: C2RustUnnamed = 129;
pub const SDL_SCANCODE_VOLUMEUP: C2RustUnnamed = 128;
pub const SDL_SCANCODE_MUTE: C2RustUnnamed = 127;
pub const SDL_SCANCODE_FIND: C2RustUnnamed = 126;
pub const SDL_SCANCODE_PASTE: C2RustUnnamed = 125;
pub const SDL_SCANCODE_COPY: C2RustUnnamed = 124;
pub const SDL_SCANCODE_CUT: C2RustUnnamed = 123;
pub const SDL_SCANCODE_UNDO: C2RustUnnamed = 122;
pub const SDL_SCANCODE_AGAIN: C2RustUnnamed = 121;
pub const SDL_SCANCODE_STOP: C2RustUnnamed = 120;
pub const SDL_SCANCODE_SELECT: C2RustUnnamed = 119;
pub const SDL_SCANCODE_MENU: C2RustUnnamed = 118;
pub const SDL_SCANCODE_HELP: C2RustUnnamed = 117;
pub const SDL_SCANCODE_EXECUTE: C2RustUnnamed = 116;
pub const SDL_SCANCODE_F24: C2RustUnnamed = 115;
pub const SDL_SCANCODE_F23: C2RustUnnamed = 114;
pub const SDL_SCANCODE_F22: C2RustUnnamed = 113;
pub const SDL_SCANCODE_F21: C2RustUnnamed = 112;
pub const SDL_SCANCODE_F20: C2RustUnnamed = 111;
pub const SDL_SCANCODE_F19: C2RustUnnamed = 110;
pub const SDL_SCANCODE_F18: C2RustUnnamed = 109;
pub const SDL_SCANCODE_F17: C2RustUnnamed = 108;
pub const SDL_SCANCODE_F16: C2RustUnnamed = 107;
pub const SDL_SCANCODE_F15: C2RustUnnamed = 106;
pub const SDL_SCANCODE_F14: C2RustUnnamed = 105;
pub const SDL_SCANCODE_F13: C2RustUnnamed = 104;
pub const SDL_SCANCODE_KP_EQUALS: C2RustUnnamed = 103;
pub const SDL_SCANCODE_POWER: C2RustUnnamed = 102;
pub const SDL_SCANCODE_APPLICATION: C2RustUnnamed = 101;
pub const SDL_SCANCODE_NONUSBACKSLASH: C2RustUnnamed = 100;
pub const SDL_SCANCODE_KP_PERIOD: C2RustUnnamed = 99;
pub const SDL_SCANCODE_KP_0: C2RustUnnamed = 98;
pub const SDL_SCANCODE_KP_9: C2RustUnnamed = 97;
pub const SDL_SCANCODE_KP_8: C2RustUnnamed = 96;
pub const SDL_SCANCODE_KP_7: C2RustUnnamed = 95;
pub const SDL_SCANCODE_KP_6: C2RustUnnamed = 94;
pub const SDL_SCANCODE_KP_5: C2RustUnnamed = 93;
pub const SDL_SCANCODE_KP_4: C2RustUnnamed = 92;
pub const SDL_SCANCODE_KP_3: C2RustUnnamed = 91;
pub const SDL_SCANCODE_KP_2: C2RustUnnamed = 90;
pub const SDL_SCANCODE_KP_1: C2RustUnnamed = 89;
pub const SDL_SCANCODE_KP_ENTER: C2RustUnnamed = 88;
pub const SDL_SCANCODE_KP_PLUS: C2RustUnnamed = 87;
pub const SDL_SCANCODE_KP_MINUS: C2RustUnnamed = 86;
pub const SDL_SCANCODE_KP_MULTIPLY: C2RustUnnamed = 85;
pub const SDL_SCANCODE_KP_DIVIDE: C2RustUnnamed = 84;
pub const SDL_SCANCODE_NUMLOCKCLEAR: C2RustUnnamed = 83;
pub const SDL_SCANCODE_UP: C2RustUnnamed = 82;
pub const SDL_SCANCODE_DOWN: C2RustUnnamed = 81;
pub const SDL_SCANCODE_LEFT: C2RustUnnamed = 80;
pub const SDL_SCANCODE_RIGHT: C2RustUnnamed = 79;
pub const SDL_SCANCODE_PAGEDOWN: C2RustUnnamed = 78;
pub const SDL_SCANCODE_END: C2RustUnnamed = 77;
pub const SDL_SCANCODE_DELETE: C2RustUnnamed = 76;
pub const SDL_SCANCODE_PAGEUP: C2RustUnnamed = 75;
pub const SDL_SCANCODE_HOME: C2RustUnnamed = 74;
pub const SDL_SCANCODE_INSERT: C2RustUnnamed = 73;
pub const SDL_SCANCODE_PAUSE: C2RustUnnamed = 72;
pub const SDL_SCANCODE_SCROLLLOCK: C2RustUnnamed = 71;
pub const SDL_SCANCODE_PRINTSCREEN: C2RustUnnamed = 70;
pub const SDL_SCANCODE_F12: C2RustUnnamed = 69;
pub const SDL_SCANCODE_F11: C2RustUnnamed = 68;
pub const SDL_SCANCODE_F10: C2RustUnnamed = 67;
pub const SDL_SCANCODE_F9: C2RustUnnamed = 66;
pub const SDL_SCANCODE_F8: C2RustUnnamed = 65;
pub const SDL_SCANCODE_F7: C2RustUnnamed = 64;
pub const SDL_SCANCODE_F6: C2RustUnnamed = 63;
pub const SDL_SCANCODE_F5: C2RustUnnamed = 62;
pub const SDL_SCANCODE_F4: C2RustUnnamed = 61;
pub const SDL_SCANCODE_F3: C2RustUnnamed = 60;
pub const SDL_SCANCODE_F2: C2RustUnnamed = 59;
pub const SDL_SCANCODE_F1: C2RustUnnamed = 58;
pub const SDL_SCANCODE_CAPSLOCK: C2RustUnnamed = 57;
pub const SDL_SCANCODE_SLASH: C2RustUnnamed = 56;
pub const SDL_SCANCODE_PERIOD: C2RustUnnamed = 55;
pub const SDL_SCANCODE_COMMA: C2RustUnnamed = 54;
pub const SDL_SCANCODE_GRAVE: C2RustUnnamed = 53;
pub const SDL_SCANCODE_APOSTROPHE: C2RustUnnamed = 52;
pub const SDL_SCANCODE_SEMICOLON: C2RustUnnamed = 51;
pub const SDL_SCANCODE_NONUSHASH: C2RustUnnamed = 50;
pub const SDL_SCANCODE_BACKSLASH: C2RustUnnamed = 49;
pub const SDL_SCANCODE_RIGHTBRACKET: C2RustUnnamed = 48;
pub const SDL_SCANCODE_LEFTBRACKET: C2RustUnnamed = 47;
pub const SDL_SCANCODE_EQUALS: C2RustUnnamed = 46;
pub const SDL_SCANCODE_MINUS: C2RustUnnamed = 45;
pub const SDL_SCANCODE_SPACE: C2RustUnnamed = 44;
pub const SDL_SCANCODE_TAB: C2RustUnnamed = 43;
pub const SDL_SCANCODE_BACKSPACE: C2RustUnnamed = 42;
pub const SDL_SCANCODE_ESCAPE: C2RustUnnamed = 41;
pub const SDL_SCANCODE_RETURN: C2RustUnnamed = 40;
pub const SDL_SCANCODE_0: C2RustUnnamed = 39;
pub const SDL_SCANCODE_9: C2RustUnnamed = 38;
pub const SDL_SCANCODE_8: C2RustUnnamed = 37;
pub const SDL_SCANCODE_7: C2RustUnnamed = 36;
pub const SDL_SCANCODE_6: C2RustUnnamed = 35;
pub const SDL_SCANCODE_5: C2RustUnnamed = 34;
pub const SDL_SCANCODE_4: C2RustUnnamed = 33;
pub const SDL_SCANCODE_3: C2RustUnnamed = 32;
pub const SDL_SCANCODE_2: C2RustUnnamed = 31;
pub const SDL_SCANCODE_1: C2RustUnnamed = 30;
pub const SDL_SCANCODE_Z: C2RustUnnamed = 29;
pub const SDL_SCANCODE_Y: C2RustUnnamed = 28;
pub const SDL_SCANCODE_X: C2RustUnnamed = 27;
pub const SDL_SCANCODE_W: C2RustUnnamed = 26;
pub const SDL_SCANCODE_V: C2RustUnnamed = 25;
pub const SDL_SCANCODE_U: C2RustUnnamed = 24;
pub const SDL_SCANCODE_T: C2RustUnnamed = 23;
pub const SDL_SCANCODE_S: C2RustUnnamed = 22;
pub const SDL_SCANCODE_R: C2RustUnnamed = 21;
pub const SDL_SCANCODE_Q: C2RustUnnamed = 20;
pub const SDL_SCANCODE_P: C2RustUnnamed = 19;
pub const SDL_SCANCODE_O: C2RustUnnamed = 18;
pub const SDL_SCANCODE_N: C2RustUnnamed = 17;
pub const SDL_SCANCODE_M: C2RustUnnamed = 16;
pub const SDL_SCANCODE_L: C2RustUnnamed = 15;
pub const SDL_SCANCODE_K: C2RustUnnamed = 14;
pub const SDL_SCANCODE_J: C2RustUnnamed = 13;
pub const SDL_SCANCODE_I: C2RustUnnamed = 12;
pub const SDL_SCANCODE_H: C2RustUnnamed = 11;
pub const SDL_SCANCODE_G: C2RustUnnamed = 10;
pub const SDL_SCANCODE_F: C2RustUnnamed = 9;
pub const SDL_SCANCODE_E: C2RustUnnamed = 8;
pub const SDL_SCANCODE_D: C2RustUnnamed = 7;
pub const SDL_SCANCODE_C: C2RustUnnamed = 6;
pub const SDL_SCANCODE_B: C2RustUnnamed = 5;
pub const SDL_SCANCODE_A: C2RustUnnamed = 4;
pub const SDL_SCANCODE_UNKNOWN: C2RustUnnamed = 0;

static mut lastAction: u64 = 0;
static mut stateLast: *mut uchar = std::ptr::null_mut();
static mut stateCurr: *mut uchar = std::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn Keyboard_Init() {
    let mut size: i32 = 0;
    let mut state: *const uchar = SDL_GetKeyboardState(&mut size);
    stateLast =
        MemAlloc((::core::mem::size_of::<uchar>()).wrapping_mul(size as usize)) as *mut uchar;
    stateCurr =
        MemAlloc((::core::mem::size_of::<uchar>()).wrapping_mul(size as usize)) as *mut uchar;
    MemCpy(
        stateLast as *mut libc::c_void,
        state as *const libc::c_void,
        size as usize,
    );
    MemCpy(
        stateCurr as *mut libc::c_void,
        state as *const libc::c_void,
        size as usize,
    );
    lastAction = SDL_GetPerformanceCounter();
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_Free() {
    MemFree(stateLast as *const libc::c_void);
    MemFree(stateCurr as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_UpdatePre() {
    let mut size: i32 = 0;
    let mut state: *const uchar = SDL_GetKeyboardState(&mut size);
    MemCpy(
        stateLast as *mut libc::c_void,
        state as *const libc::c_void,
        size as usize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_UpdatePost() {
    let mut size: i32 = 0;
    let mut state: *const uchar = SDL_GetKeyboardState(&mut size);
    MemCpy(
        stateCurr as *mut libc::c_void,
        state as *const libc::c_void,
        size as usize,
    );
    let mut i: i32 = 0 as i32;
    while i < size {
        if *stateCurr.offset(i as isize) as i32 != *stateLast.offset(i as isize) as i32 {
            lastAction = SDL_GetPerformanceCounter();
            break;
        } else {
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_Down(mut key: Key) -> bool {
    return *stateCurr.offset(key as isize) as i32 != 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_Pressed(mut key: Key) -> bool {
    return *stateCurr.offset(key as isize) as i32 != 0 && *stateLast.offset(key as isize) == 0;
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_Released(mut key: Key) -> bool {
    return *stateCurr.offset(key as isize) == 0 && *stateLast.offset(key as isize) as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Keyboard_GetIdleTime() -> f64 {
    let mut now: u64 = SDL_GetPerformanceCounter();
    return now.wrapping_sub(lastAction) as f64 / SDL_GetPerformanceFrequency() as f64;
}
#[no_mangle]
pub unsafe extern "C" fn KeyMod_Alt() -> bool {
    return *stateCurr.offset(SDL_SCANCODE_LALT as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_SCANCODE_RALT as i32 as isize) as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn KeyMod_Ctrl() -> bool {
    return *stateCurr.offset(SDL_SCANCODE_LCTRL as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_SCANCODE_RCTRL as i32 as isize) as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn KeyMod_Shift() -> bool {
    return *stateCurr.offset(SDL_SCANCODE_LSHIFT as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_SCANCODE_RSHIFT as i32 as isize) as i32 != 0;
}
