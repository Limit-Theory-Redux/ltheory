use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
use crate::Button::*;
use crate::Modifier::*;
use crate::DeviceType::*;
use crate::ResourceType::*;
use crate::State::*;

extern "C" {
    pub type __sFILEX;
    pub type SDL_Window;
    pub type _SDL_Joystick;
    pub type _SDL_GameController;
    pub type SDL_SysWMmsg;
    fn memmove(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
    fn SDL_RWFromFile(
        file: *const libc::c_char,
        mode: *const libc::c_char,
    ) -> *mut SDL_RWops;
    fn Button_ToSDLControllerButton(_: Button) -> SDL_GameControllerButton;
    fn Button_FromSDLControllerButton(_: SDL_GameControllerButton) -> Button;
    fn Button_ToSDLControllerAxis(_: Button) -> SDL_GameControllerAxis;
    fn Button_FromSDLControllerAxis(_: SDL_GameControllerAxis) -> Button;
    fn SDL_WarpMouseInWindow(window: *mut SDL_Window, x: libc::c_int, y: libc::c_int);
    fn SDL_CaptureMouse(enabled: SDL_bool) -> libc::c_int;
    fn SDL_ShowCursor(toggle: libc::c_int) -> libc::c_int;
    fn SDL_JoystickInstanceID(joystick: *mut SDL_Joystick) -> SDL_JoystickID;
    fn SDL_GameControllerAddMappingsFromRW(
        rw: *mut SDL_RWops,
        freerw: libc::c_int,
    ) -> libc::c_int;
    fn SDL_IsGameController(joystick_index: libc::c_int) -> SDL_bool;
    fn SDL_GameControllerOpen(joystick_index: libc::c_int) -> *mut SDL_GameController;
    fn SDL_GameControllerFromInstanceID(
        joyid: SDL_JoystickID,
    ) -> *mut SDL_GameController;
    fn SDL_GameControllerGetJoystick(
        gamecontroller: *mut SDL_GameController,
    ) -> *mut SDL_Joystick;
    fn SDL_GameControllerGetAxis(
        gamecontroller: *mut SDL_GameController,
        axis: SDL_GameControllerAxis,
    ) -> Sint16;
    fn SDL_GameControllerGetButton(
        gamecontroller: *mut SDL_GameController,
        button: SDL_GameControllerButton,
    ) -> Uint8;
    fn SDL_GameControllerClose(gamecontroller: *mut SDL_GameController);
    fn SDL_PollEvent(event: *mut SDL_Event) -> libc::c_int;
    fn SDL_SetHint(name: *const libc::c_char, value: *const libc::c_char) -> SDL_bool;
    fn SDL_GetTicks() -> Uint32;
    fn Button_ToDeviceType(_: Button) -> DeviceType;
    fn Button_IsAutoRelease(_: Button) -> bool;
    fn Button_FromSDLScancode(_: SDL_Scancode) -> Button;
    fn Button_FromSDLMouseButton(_: uint8) -> Button;
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
}
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type int64_t = libc::c_longlong;
pub type uint8_t = libc::c_uchar;
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint8 = uint8_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: uint32,
}
pub type DeviceType = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: uint32,
    pub device: Device,
    pub button: Button,
    pub value: f32,
    pub state: State,
}
pub type State = int32;
pub type Button = int32;

pub type Modifier = int32;
pub type ResourceType = int32;
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type FILE = __sFILE;
pub type SDL_bool = libc::c_uint;
pub const SDL_TRUE: SDL_bool = 1;
pub const SDL_FALSE: SDL_bool = 0;
pub type Uint8 = uint8_t;
pub type Sint16 = int16_t;
pub type Uint16 = uint16_t;
pub type Sint32 = int32_t;
pub type Uint32 = uint32_t;
pub type Sint64 = int64_t;
pub type Uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_RWops {
    pub size: Option::<unsafe extern "C" fn(*mut SDL_RWops) -> Sint64>,
    pub seek: Option::<
        unsafe extern "C" fn(*mut SDL_RWops, Sint64, libc::c_int) -> Sint64,
    >,
    pub read: Option::<
        unsafe extern "C" fn(*mut SDL_RWops, *mut libc::c_void, libc::size_t, libc::size_t) -> libc::size_t,
    >,
    pub write: Option::<
        unsafe extern "C" fn(
            *mut SDL_RWops,
            *const libc::c_void,
            libc::size_t,
            libc::size_t,
        ) -> libc::size_t,
    >,
    pub close: Option::<unsafe extern "C" fn(*mut SDL_RWops) -> libc::c_int>,
    pub type_0: Uint32,
    pub hidden: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub stdio: C2RustUnnamed_2,
    pub mem: C2RustUnnamed_1,
    pub unknown: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub data1: *mut libc::c_void,
    pub data2: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub base: *mut Uint8,
    pub here: *mut Uint8,
    pub stop: *mut Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub autoclose: SDL_bool,
    pub fp: *mut FILE,
}
pub type C2RustUnnamed_3 = libc::c_uint;
pub const SDL_WINDOWEVENT_DISPLAY_CHANGED: C2RustUnnamed_3 = 18;
pub const SDL_WINDOWEVENT_ICCPROF_CHANGED: C2RustUnnamed_3 = 17;
pub const SDL_WINDOWEVENT_HIT_TEST: C2RustUnnamed_3 = 16;
pub const SDL_WINDOWEVENT_TAKE_FOCUS: C2RustUnnamed_3 = 15;
pub const SDL_WINDOWEVENT_CLOSE: C2RustUnnamed_3 = 14;
pub const SDL_WINDOWEVENT_FOCUS_LOST: C2RustUnnamed_3 = 13;
pub const SDL_WINDOWEVENT_FOCUS_GAINED: C2RustUnnamed_3 = 12;
pub const SDL_WINDOWEVENT_LEAVE: C2RustUnnamed_3 = 11;
pub const SDL_WINDOWEVENT_ENTER: C2RustUnnamed_3 = 10;
pub const SDL_WINDOWEVENT_RESTORED: C2RustUnnamed_3 = 9;
pub const SDL_WINDOWEVENT_MAXIMIZED: C2RustUnnamed_3 = 8;
pub const SDL_WINDOWEVENT_MINIMIZED: C2RustUnnamed_3 = 7;
pub const SDL_WINDOWEVENT_SIZE_CHANGED: C2RustUnnamed_3 = 6;
pub const SDL_WINDOWEVENT_RESIZED: C2RustUnnamed_3 = 5;
pub const SDL_WINDOWEVENT_MOVED: C2RustUnnamed_3 = 4;
pub const SDL_WINDOWEVENT_EXPOSED: C2RustUnnamed_3 = 3;
pub const SDL_WINDOWEVENT_HIDDEN: C2RustUnnamed_3 = 2;
pub const SDL_WINDOWEVENT_SHOWN: C2RustUnnamed_3 = 1;
pub const SDL_WINDOWEVENT_NONE: C2RustUnnamed_3 = 0;
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
pub type SDL_Keycode = Sint32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_Keysym {
    pub scancode: SDL_Scancode,
    pub sym: SDL_Keycode,
    pub mod_0: Uint16,
    pub unused: Uint32,
}
pub type SDL_Joystick = _SDL_Joystick;
pub type SDL_JoystickID = Sint32;
pub type SDL_JoystickPowerLevel = libc::c_int;
pub const SDL_JOYSTICK_POWER_MAX: SDL_JoystickPowerLevel = 5;
pub const SDL_JOYSTICK_POWER_WIRED: SDL_JoystickPowerLevel = 4;
pub const SDL_JOYSTICK_POWER_FULL: SDL_JoystickPowerLevel = 3;
pub const SDL_JOYSTICK_POWER_MEDIUM: SDL_JoystickPowerLevel = 2;
pub const SDL_JOYSTICK_POWER_LOW: SDL_JoystickPowerLevel = 1;
pub const SDL_JOYSTICK_POWER_EMPTY: SDL_JoystickPowerLevel = 0;
pub const SDL_JOYSTICK_POWER_UNKNOWN: SDL_JoystickPowerLevel = -1;
pub type SDL_GameController = _SDL_GameController;
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
pub type SDL_TouchID = Sint64;
pub type SDL_FingerID = Sint64;
pub type SDL_GestureID = Sint64;
pub type C2RustUnnamed_4 = libc::c_uint;
pub const SDL_LASTEVENT: C2RustUnnamed_4 = 65535;
pub const SDL_USEREVENT: C2RustUnnamed_4 = 32768;
pub const SDL_POLLSENTINEL: C2RustUnnamed_4 = 32512;
pub const SDL_RENDER_DEVICE_RESET: C2RustUnnamed_4 = 8193;
pub const SDL_RENDER_TARGETS_RESET: C2RustUnnamed_4 = 8192;
pub const SDL_SENSORUPDATE: C2RustUnnamed_4 = 4608;
pub const SDL_AUDIODEVICEREMOVED: C2RustUnnamed_4 = 4353;
pub const SDL_AUDIODEVICEADDED: C2RustUnnamed_4 = 4352;
pub const SDL_DROPCOMPLETE: C2RustUnnamed_4 = 4099;
pub const SDL_DROPBEGIN: C2RustUnnamed_4 = 4098;
pub const SDL_DROPTEXT: C2RustUnnamed_4 = 4097;
pub const SDL_DROPFILE: C2RustUnnamed_4 = 4096;
pub const SDL_CLIPBOARDUPDATE: C2RustUnnamed_4 = 2304;
pub const SDL_MULTIGESTURE: C2RustUnnamed_4 = 2050;
pub const SDL_DOLLARRECORD: C2RustUnnamed_4 = 2049;
pub const SDL_DOLLARGESTURE: C2RustUnnamed_4 = 2048;
pub const SDL_FINGERMOTION: C2RustUnnamed_4 = 1794;
pub const SDL_FINGERUP: C2RustUnnamed_4 = 1793;
pub const SDL_FINGERDOWN: C2RustUnnamed_4 = 1792;
pub const SDL_CONTROLLERSENSORUPDATE: C2RustUnnamed_4 = 1625;
pub const SDL_CONTROLLERTOUCHPADUP: C2RustUnnamed_4 = 1624;
pub const SDL_CONTROLLERTOUCHPADMOTION: C2RustUnnamed_4 = 1623;
pub const SDL_CONTROLLERTOUCHPADDOWN: C2RustUnnamed_4 = 1622;
pub const SDL_CONTROLLERDEVICEREMAPPED: C2RustUnnamed_4 = 1621;
pub const SDL_CONTROLLERDEVICEREMOVED: C2RustUnnamed_4 = 1620;
pub const SDL_CONTROLLERDEVICEADDED: C2RustUnnamed_4 = 1619;
pub const SDL_CONTROLLERBUTTONUP: C2RustUnnamed_4 = 1618;
pub const SDL_CONTROLLERBUTTONDOWN: C2RustUnnamed_4 = 1617;
pub const SDL_CONTROLLERAXISMOTION: C2RustUnnamed_4 = 1616;
pub const SDL_JOYBATTERYUPDATED: C2RustUnnamed_4 = 1543;
pub const SDL_JOYDEVICEREMOVED: C2RustUnnamed_4 = 1542;
pub const SDL_JOYDEVICEADDED: C2RustUnnamed_4 = 1541;
pub const SDL_JOYBUTTONUP: C2RustUnnamed_4 = 1540;
pub const SDL_JOYBUTTONDOWN: C2RustUnnamed_4 = 1539;
pub const SDL_JOYHATMOTION: C2RustUnnamed_4 = 1538;
pub const SDL_JOYBALLMOTION: C2RustUnnamed_4 = 1537;
pub const SDL_JOYAXISMOTION: C2RustUnnamed_4 = 1536;
pub const SDL_MOUSEWHEEL: C2RustUnnamed_4 = 1027;
pub const SDL_MOUSEBUTTONUP: C2RustUnnamed_4 = 1026;
pub const SDL_MOUSEBUTTONDOWN: C2RustUnnamed_4 = 1025;
pub const SDL_MOUSEMOTION: C2RustUnnamed_4 = 1024;
pub const SDL_TEXTEDITING_EXT: C2RustUnnamed_4 = 773;
pub const SDL_KEYMAPCHANGED: C2RustUnnamed_4 = 772;
pub const SDL_TEXTINPUT: C2RustUnnamed_4 = 771;
pub const SDL_TEXTEDITING: C2RustUnnamed_4 = 770;
pub const SDL_KEYUP: C2RustUnnamed_4 = 769;
pub const SDL_KEYDOWN: C2RustUnnamed_4 = 768;
pub const SDL_SYSWMEVENT: C2RustUnnamed_4 = 513;
pub const SDL_WINDOWEVENT: C2RustUnnamed_4 = 512;
pub const SDL_DISPLAYEVENT: C2RustUnnamed_4 = 336;
pub const SDL_LOCALECHANGED: C2RustUnnamed_4 = 263;
pub const SDL_APP_DIDENTERFOREGROUND: C2RustUnnamed_4 = 262;
pub const SDL_APP_WILLENTERFOREGROUND: C2RustUnnamed_4 = 261;
pub const SDL_APP_DIDENTERBACKGROUND: C2RustUnnamed_4 = 260;
pub const SDL_APP_WILLENTERBACKGROUND: C2RustUnnamed_4 = 259;
pub const SDL_APP_LOWMEMORY: C2RustUnnamed_4 = 258;
pub const SDL_APP_TERMINATING: C2RustUnnamed_4 = 257;
pub const SDL_QUIT: C2RustUnnamed_4 = 256;
pub const SDL_FIRSTEVENT: C2RustUnnamed_4 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_CommonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DisplayEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub display: Uint32,
    pub event: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub data1: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_WindowEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub event: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub data1: Sint32,
    pub data2: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_KeyboardEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub state: Uint8,
    pub repeat: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub keysym: SDL_Keysym,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextEditingEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: [libc::c_char; 32],
    pub start: Sint32,
    pub length: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextEditingExtEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: *mut libc::c_char,
    pub start: Sint32,
    pub length: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TextInputEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub text: [libc::c_char; 32],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseMotionEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub state: Uint32,
    pub x: Sint32,
    pub y: Sint32,
    pub xrel: Sint32,
    pub yrel: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub button: Uint8,
    pub state: Uint8,
    pub clicks: Uint8,
    pub padding1: Uint8,
    pub x: Sint32,
    pub y: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MouseWheelEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub which: Uint32,
    pub x: Sint32,
    pub y: Sint32,
    pub direction: Uint32,
    pub preciseX: f32,
    pub preciseY: f32,
    pub mouseX: Sint32,
    pub mouseY: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyAxisEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub axis: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub value: Sint16,
    pub padding4: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyBallEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub ball: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub xrel: Sint16,
    pub yrel: Sint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyHatEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub hat: Uint8,
    pub value: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub button: Uint8,
    pub state: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoyBatteryEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub level: SDL_JoystickPowerLevel,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerAxisEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub axis: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub value: Sint16,
    pub padding4: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerButtonEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub button: Uint8,
    pub state: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerTouchpadEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub touchpad: Sint32,
    pub finger: Sint32,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_ControllerSensorEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: SDL_JoystickID,
    pub sensor: Sint32,
    pub data: [f32; 3],
    pub timestamp_us: Uint64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_AudioDeviceEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Uint32,
    pub iscapture: Uint8,
    pub padding1: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_TouchFingerEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub fingerId: SDL_FingerID,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub pressure: f32,
    pub windowID: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_MultiGestureEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub dTheta: f32,
    pub dDist: f32,
    pub x: f32,
    pub y: f32,
    pub numFingers: Uint16,
    pub padding: Uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DollarGestureEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub touchId: SDL_TouchID,
    pub gestureId: SDL_GestureID,
    pub numFingers: Uint32,
    pub error: f32,
    pub x: f32,
    pub y: f32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_DropEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub file: *mut libc::c_char,
    pub windowID: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_SensorEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub which: Sint32,
    pub data: [f32; 6],
    pub timestamp_us: Uint64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_QuitEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_UserEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub windowID: Uint32,
    pub code: Sint32,
    pub data1: *mut libc::c_void,
    pub data2: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_SysWMEvent {
    pub type_0: Uint32,
    pub timestamp: Uint32,
    pub msg: *mut SDL_SysWMmsg,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SDL_Event {
    pub type_0: Uint32,
    pub common: SDL_CommonEvent,
    pub display: SDL_DisplayEvent,
    pub window: SDL_WindowEvent,
    pub key: SDL_KeyboardEvent,
    pub edit: SDL_TextEditingEvent,
    pub editExt: SDL_TextEditingExtEvent,
    pub text: SDL_TextInputEvent,
    pub motion: SDL_MouseMotionEvent,
    pub button: SDL_MouseButtonEvent,
    pub wheel: SDL_MouseWheelEvent,
    pub jaxis: SDL_JoyAxisEvent,
    pub jball: SDL_JoyBallEvent,
    pub jhat: SDL_JoyHatEvent,
    pub jbutton: SDL_JoyButtonEvent,
    pub jdevice: SDL_JoyDeviceEvent,
    pub jbattery: SDL_JoyBatteryEvent,
    pub caxis: SDL_ControllerAxisEvent,
    pub cbutton: SDL_ControllerButtonEvent,
    pub cdevice: SDL_ControllerDeviceEvent,
    pub ctouchpad: SDL_ControllerTouchpadEvent,
    pub csensor: SDL_ControllerSensorEvent,
    pub adevice: SDL_AudioDeviceEvent,
    pub sensor: SDL_SensorEvent,
    pub quit: SDL_QuitEvent,
    pub user: SDL_UserEvent,
    pub syswm: SDL_SysWMEvent,
    pub tfinger: SDL_TouchFingerEvent,
    pub mgesture: SDL_MultiGestureEvent,
    pub dgesture: SDL_DollarGestureEvent,
    pub drop: SDL_DropEvent,
    pub padding: [Uint8; 56],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DeviceState {
    pub transitions: [int32; 512],
    pub buttons: [bool; 512],
    pub axes: [f32; 512],
    pub lastEventTimestamp: uint32,
    pub isConnected: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DeviceList {
    pub devices_size: int32,
    pub devices_capacity: int32,
    pub devices_data: *mut DeviceState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub activeDevice: Device,
    pub lastTimestamp: uint32,
    pub lastEventTimestamp: uint32,
    pub lastMousePosition: IVec2,
    pub autoHideMouse: bool,
    pub deviceLists: [DeviceList; 4],
    pub events_size: int32,
    pub events_capacity: int32,
    pub events_data: *mut InputEvent,
    pub downButtons_size: int32,
    pub downButtons_capacity: int32,
    pub downButtons_data: *mut InputEvent,
    pub autoRelease_size: int32,
    pub autoRelease_capacity: int32,
    pub autoRelease_data: *mut InputEvent,
    pub injectedEvents_size: int32,
    pub injectedEvents_capacity: int32,
    pub injectedEvents_data: *mut InputEvent,
}

#[inline]
unsafe extern "C" fn Clamp(
    mut t: f64,
    mut lower: f64,
    mut upper: f64,
) -> f64 {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}

static mut Threshold_Pressed: f32 = 0.5f32;
static mut Threshold_Released: f32 = 0.4f32;
static mut this: Input = {
    let mut init = Input {
        activeDevice: {
            let mut init = Device {
                type_0: 0 as libc::c_int,
                id: 0,
            };
            init
        },
        lastTimestamp: 0,
        lastEventTimestamp: 0,
        lastMousePosition: IVec2 { x: 0, y: 0 },
        autoHideMouse: false,
        deviceLists: [DeviceList {
            devices_size: 0,
            devices_capacity: 0,
            devices_data: 0 as *const DeviceState as *mut DeviceState,
        }; 4],
        events_size: 0,
        events_capacity: 0,
        events_data: 0 as *const InputEvent as *mut InputEvent,
        downButtons_size: 0,
        downButtons_capacity: 0,
        downButtons_data: 0 as *const InputEvent as *mut InputEvent,
        autoRelease_size: 0,
        autoRelease_capacity: 0,
        autoRelease_data: 0 as *const InputEvent as *mut InputEvent,
        injectedEvents_size: 0,
        injectedEvents_capacity: 0,
        injectedEvents_data: 0 as *const InputEvent as *mut InputEvent,
    };
    init
};
#[inline]
unsafe extern "C" fn Input_EnsureDeviceState(mut device: Device) -> *mut DeviceState {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    while (*deviceList).devices_size as uint32 <= device.id {
        let mut deviceState: DeviceState = {
            let mut init = DeviceState {
                transitions: [0; 512],
                buttons: [false; 512],
                axes: [0.; 512],
                lastEventTimestamp: 0,
                isConnected: false,
            };
            init
        };
        if ((*deviceList).devices_capacity == (*deviceList).devices_size) as libc::c_int
            as libc::c_long != 0
        {
            (*deviceList)
                .devices_capacity = if (*deviceList).devices_capacity != 0 {
                (*deviceList).devices_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize: usize = ::core::mem::size_of::<DeviceState>();
            let mut pData: *mut *mut libc::c_void = &mut (*deviceList).devices_data
                as *mut *mut DeviceState as *mut *mut libc::c_void;
            *pData = MemRealloc(
                (*deviceList).devices_data as *mut libc::c_void,
                ((*deviceList).devices_capacity as usize).wrapping_mul(elemSize as usize),
            );
        }
        let fresh0 = (*deviceList).devices_size;
        (*deviceList).devices_size = (*deviceList).devices_size + 1;
        *((*deviceList).devices_data).offset(fresh0 as isize) = deviceState;
    }
    return ((*deviceList).devices_data).offset(device.id as isize);
}
#[inline]
unsafe extern "C" fn Input_GetDeviceState(mut device: Device) -> *mut DeviceState {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    return ((*deviceList).devices_data).offset(device.id as isize);
}
#[inline]
unsafe extern "C" fn Input_SetActiveDevice(mut device: Device) {
    this.activeDevice = device;
    if this.autoHideMouse {
        SDL_ShowCursor(
            if device.type_0 == DeviceType_Mouse {
                1 as libc::c_int
            } else {
                0 as libc::c_int
            },
        );
    }
}
#[inline]
unsafe extern "C" fn Input_GetDeviceExists(mut device: Device) -> bool {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    if device.id < (*deviceList).devices_size as uint32 {
        let mut deviceState: *mut DeviceState = ((*deviceList).devices_data)
            .offset(device.id as isize);
        return (*deviceState).isConnected;
    }
    return 0 as libc::c_int != 0;
}
#[inline]
unsafe extern "C" fn Input_GetDevicePressedImpl(
    mut device: Device,
    mut button: Button,
) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return if (*deviceState).buttons[button as usize] as libc::c_int != 0 {
        ((*deviceState).transitions[button as usize] > 0 as libc::c_int) as libc::c_int
    } else {
        ((*deviceState).transitions[button as usize] > 1 as libc::c_int) as libc::c_int
    } != 0;
}
#[inline]
unsafe extern "C" fn Input_GetDeviceDownImpl(
    mut device: Device,
    mut button: Button,
) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (*deviceState).buttons[button as usize] as libc::c_int != 0
        || (*deviceState).transitions[button as usize] > 0 as libc::c_int;
}
#[inline]
unsafe extern "C" fn Input_GetDeviceReleasedImpl(
    mut device: Device,
    mut button: Button,
) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return if (*deviceState).buttons[button as usize] as libc::c_int != 0 {
        ((*deviceState).transitions[button as usize] > 1 as libc::c_int) as libc::c_int
    } else {
        ((*deviceState).transitions[button as usize] > 0 as libc::c_int) as libc::c_int
    } != 0;
}
#[inline]
unsafe extern "C" fn Input_GetDeviceValueImpl(
    mut device: Device,
    mut button: Button,
) -> f32 {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (*deviceState).axes[button as usize];
}
#[inline]
unsafe extern "C" fn Input_GetDeviceIdleTimeImpl(mut device: Device) -> f32 {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (this.lastTimestamp).wrapping_sub((*deviceState).lastEventTimestamp)
        as f32 / 1000.0f32;
}
#[inline]
unsafe extern "C" fn Input_DetermineButtonState(mut event: InputEvent) -> State {
    let mut buttonState: State = State_Null;
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    let mut down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.value > Threshold_Pressed {
        buttonState |= State_Pressed | State_Down;
    }
    if down as libc::c_int != 0 && event.value < Threshold_Released {
        buttonState |= State_Released;
    }
    return buttonState;
}
#[inline]
unsafe extern "C" fn Input_AppendEvent(mut event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    if (this.events_capacity == this.events_size) as libc::c_long != 0
    {
        this
            .events_capacity = if this.events_capacity != 0 {
            this.events_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<InputEvent>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut this.events_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.events_data as *mut libc::c_void,
            (this.events_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh1 = this.events_size;
    this.events_size = this.events_size + 1;
    *(this.events_data).offset(fresh1 as isize) = event;
}
#[inline]
unsafe extern "C" fn Input_InjectEvent(mut event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    if (this.injectedEvents_capacity == this.injectedEvents_size) as libc::c_int
        as libc::c_long != 0
    {
        this
            .injectedEvents_capacity = if this.injectedEvents_capacity != 0 {
            this.injectedEvents_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<InputEvent>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut this.injectedEvents_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.injectedEvents_data as *mut libc::c_void,
            (this.injectedEvents_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh2 = this.injectedEvents_size;
    this.injectedEvents_size = this.injectedEvents_size + 1;
    *(this.injectedEvents_data).offset(fresh2 as isize) = event;
}
#[inline]
unsafe extern "C" fn Input_SetButton(mut event: InputEvent) {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    (*deviceState).axes[event.button as usize] = event.value;
    let mut down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.state & State_Pressed == State_Pressed {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = 1 as libc::c_int != 0;
        if (this.downButtons_capacity == this.downButtons_size) as libc::c_int
            as libc::c_long != 0
        {
            this
                .downButtons_capacity = if this.downButtons_capacity != 0 {
                this.downButtons_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize: usize = ::core::mem::size_of::<InputEvent>();
            let mut pData: *mut *mut libc::c_void = &mut this.downButtons_data
                as *mut *mut InputEvent as *mut *mut libc::c_void;
            *pData = MemRealloc(
                this.downButtons_data as *mut libc::c_void,
                (this.downButtons_capacity as usize).wrapping_mul(elemSize as usize),
            );
        }
        let fresh3 = this.downButtons_size;
        this.downButtons_size = this.downButtons_size + 1;
        *(this.downButtons_data).offset(fresh3 as isize) = event;
        if event.device.type_0 != DeviceType_Null {
            Input_SetActiveDevice(event.device);
        }
    }
    if down as libc::c_int != 0 && event.state & State_Released == State_Released {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = 0 as libc::c_int != 0;
        let mut i: libc::c_int = this.downButtons_size - 1 as libc::c_int;
        while i >= 0 as libc::c_int {
            if (*(this.downButtons_data).offset(i as isize)).button == event.button {
                if i != this.downButtons_size - 1 as libc::c_int {
                    let mut curr: *mut libc::c_void = (this.downButtons_data)
                        .offset(i as isize)
                        .offset(0) as *mut libc::c_void;
                    let mut next: *mut libc::c_void = (this.downButtons_data)
                        .offset(i as isize)
                        .offset(1) as *mut libc::c_void;
                    let mut elemSize_0: usize = ::core::mem::size_of::<InputEvent>();
                    MemMove(
                        curr,
                        next,
                        ((this.downButtons_size - 1 as libc::c_int - i)
                            as usize).wrapping_mul(elemSize_0),
                    );
                }
                this.downButtons_size -= 1;
            }
            i -= 1;
        }
    }
    if Button_IsAutoRelease(event.button) {
        if (this.autoRelease_capacity == this.autoRelease_size) as libc::c_int
            as libc::c_long != 0
        {
            this
                .autoRelease_capacity = if this.autoRelease_capacity != 0 {
                this.autoRelease_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            let mut elemSize_1: usize = ::core::mem::size_of::<InputEvent>();
            let mut pData_0: *mut *mut libc::c_void = &mut this.autoRelease_data
                as *mut *mut InputEvent as *mut *mut libc::c_void;
            *pData_0 = MemRealloc(
                this.autoRelease_data as *mut libc::c_void,
                (this.autoRelease_capacity as usize).wrapping_mul(elemSize_1 as usize),
            );
        }
        let fresh4 = this.autoRelease_size;
        this.autoRelease_size = this.autoRelease_size + 1;
        *(this.autoRelease_data).offset(fresh4 as isize) = event;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Input_Init() {
    let mut result: SDL_bool = SDL_SetHint(
        b"SDL_MOUSE_FOCUS_CLICKTHROUGH\0" as *const u8 as *const libc::c_char,
        b"1\0" as *const u8 as *const libc::c_char,
    );
    if result as libc::c_uint != SDL_TRUE as libc::c_int as libc::c_uint {
        Warn(b"Input_Init: SDL_SetHint failed\0" as *const u8 as *const libc::c_char);
    }
    let mut iDev: libc::c_int = 0 as libc::c_int;
    while iDev < 4 as libc::c_int {
        let mut device: Device = {
            let mut init = Device {
                type_0: iDev,
                id: 0 as libc::c_int as uint32,
            };
            init
        };
        let mut deviceState: *mut DeviceState = Input_EnsureDeviceState(device);
        (*deviceState).isConnected = iDev != DeviceType_Gamepad;
        iDev += 1;
    }
    if (this.events_capacity < 16 as libc::c_int) as libc::c_long != 0 {
        this.events_capacity = 16 as libc::c_int;
        let mut elemSize: usize = ::core::mem::size_of::<InputEvent>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut this.events_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.events_data as *mut libc::c_void,
            (this.events_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if (this.downButtons_capacity < 16 as libc::c_int) as libc::c_long
        != 0
    {
        this.downButtons_capacity = 16 as libc::c_int;
        let mut elemSize_0: usize = ::core::mem::size_of::<InputEvent>();
        let mut pData_0: *mut *mut libc::c_void = &mut this.downButtons_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            this.downButtons_data as *mut libc::c_void,
            (this.downButtons_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
    if (this.autoRelease_capacity < 16 as libc::c_int) as libc::c_long
        != 0
    {
        this.autoRelease_capacity = 16 as libc::c_int;
        let mut elemSize_1: usize = ::core::mem::size_of::<InputEvent>();
        let mut pData_1: *mut *mut libc::c_void = &mut this.autoRelease_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_1 = MemRealloc(
            this.autoRelease_data as *mut libc::c_void,
            (this.autoRelease_capacity as usize).wrapping_mul(elemSize_1 as usize),
        );
    }
    if (this.injectedEvents_capacity < 16 as libc::c_int) as libc::c_int
        as libc::c_long != 0
    {
        this.injectedEvents_capacity = 16 as libc::c_int;
        let mut elemSize_2: usize = ::core::mem::size_of::<InputEvent>();
        let mut pData_2: *mut *mut libc::c_void = &mut this.injectedEvents_data
            as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_2 = MemRealloc(
            this.injectedEvents_data as *mut libc::c_void,
            (this.injectedEvents_capacity as usize).wrapping_mul(elemSize_2 as usize),
        );
    }
    let mut device_0: Device = {
        let mut init = Device {
            type_0: DeviceType_Mouse,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    Input_SetActiveDevice(device_0);
}
#[no_mangle]
pub unsafe extern "C" fn Input_Free() {
    let mut iDev: libc::c_int = 0 as libc::c_int;
    while iDev < 4 as libc::c_int {
        MemFree(this.deviceLists[iDev as usize].devices_data as *const libc::c_void);
        iDev += 1;
    }
    MemFree(this.events_data as *const libc::c_void);
    MemFree(this.downButtons_data as *const libc::c_void);
    MemFree(this.autoRelease_data as *const libc::c_void);
    MemFree(this.injectedEvents_data as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Input_Update() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Input_Update\0"))
            .as_ptr(),
    );
    this.lastTimestamp = SDL_GetTicks();
    this.lastMousePosition.x = Input_GetValue(Button_Mouse_X) as libc::c_int;
    this.lastMousePosition.y = Input_GetValue(Button_Mouse_Y) as libc::c_int;
    let mut iDev: libc::c_int = 0 as libc::c_int;
    while iDev < 4 as libc::c_int {
        let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
            .as_mut_ptr()
            .offset(iDev as isize) as *mut DeviceList;
        let mut deviceState: *mut DeviceState = (*deviceList).devices_data;
        let mut __iterend: *mut DeviceState = ((*deviceList).devices_data)
            .offset((*deviceList).devices_size as isize);
        while deviceState < __iterend {
            MemSet(
                ((*deviceState).transitions).as_mut_ptr() as *mut libc::c_void,
                0 as libc::c_int,
                ::core::mem::size_of::<[int32; 512]>(),
            );
            deviceState = deviceState.offset(1);
        }
        iDev += 1;
    }
    this.events_size = 0 as libc::c_int;
    let mut event: *mut InputEvent = this.injectedEvents_data;
    let mut __iterend_0: *mut InputEvent = (this.injectedEvents_data)
        .offset(this.injectedEvents_size as isize);
    while event < __iterend_0 {
        Input_AppendEvent(*event);
        event = event.offset(1);
    }
    this.injectedEvents_size = 0 as libc::c_int;
    let mut down: *mut InputEvent = this.autoRelease_data;
    let mut __iterend_1: *mut InputEvent = (this.autoRelease_data)
        .offset(this.autoRelease_size as isize);
    while down < __iterend_1 {
        let mut deviceState_0: *mut DeviceState = Input_GetDeviceState((*down).device);
        if (*deviceState_0).axes[(*down).button as usize] != 0.0f32 {
            (*down).value = 0.0f32;
            (*down).state = State_Changed | Input_DetermineButtonState(*down);
            (*down).timestamp = SDL_GetTicks();
            Input_SetButton(*down);
            Input_AppendEvent(*down);
        }
        down = down.offset(1);
    }
    let mut down_0: *mut InputEvent = this.downButtons_data;
    let mut __iterend_2: *mut InputEvent = (this.downButtons_data)
        .offset(this.downButtons_size as isize);
    while down_0 < __iterend_2 {
        let mut deviceState_1: *mut DeviceState = Input_GetDeviceState((*down_0).device);
        (*down_0).value = (*deviceState_1).axes[(*down_0).button as usize];
        (*down_0).state = State_Down;
        (*down_0).timestamp = SDL_GetTicks();
        Input_AppendEvent(*down_0);
        down_0 = down_0.offset(1);
    }
    let mut sdl: SDL_Event = SDL_Event { type_0: 0 };
    while SDL_PollEvent(&mut sdl) != 0 as libc::c_int {
        let mut event_0: InputEvent = {
            let mut init = InputEvent {
                timestamp: 0,
                device: Device { type_0: 0, id: 0 },
                button: 0,
                value: 0.,
                state: 0,
            };
            init
        };
        event_0.timestamp = sdl.common.timestamp;
        match sdl.type_0 {
            768 => {
                if sdl.key.repeat != 0 {
                    continue;
                }
                let mut device: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Keyboard,
                        id: 0 as libc::c_int as uint32,
                    };
                    init
                };
                event_0.device = device;
                event_0.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                if event_0.button == Button_Null {
                    continue;
                }
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            769 => {
                let mut device_0: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Keyboard,
                        id: 0 as libc::c_int as uint32,
                    };
                    init
                };
                event_0.device = device_0;
                event_0.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                if event_0.button == Button_Null {
                    continue;
                }
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1025 => {
                let mut device_1: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Mouse,
                        id: sdl.button.which,
                    };
                    init
                };
                event_0.device = device_1;
                event_0.button = Button_FromSDLMouseButton(sdl.button.button);
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_EnsureDeviceState(event_0.device);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1026 => {
                let mut device_2: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Mouse,
                        id: sdl.button.which,
                    };
                    init
                };
                event_0.device = device_2;
                event_0.button = Button_FromSDLMouseButton(sdl.button.button);
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                Input_EnsureDeviceState(event_0.device);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1024 => {
                let mut device_3: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Mouse,
                        id: sdl.motion.which,
                    };
                    init
                };
                let mut deviceState_2: *mut DeviceState = Input_EnsureDeviceState(
                    device_3,
                );
                event_0.device = device_3;
                event_0.button = Button_Mouse_X;
                event_0.value = sdl.motion.x as f32;
                event_0.state = State_Changed;
                if event_0.value != (*deviceState_2).axes[event_0.button as usize] {
                    (*deviceState_2).axes[event_0.button as usize] = event_0.value;
                    Input_SetActiveDevice(event_0.device);
                    Input_AppendEvent(event_0);
                }
                event_0.device = device_3;
                event_0.button = Button_Mouse_Y;
                event_0.value = sdl.motion.y as f32;
                event_0.state = State_Changed;
                if event_0.value != (*deviceState_2).axes[event_0.button as usize] {
                    (*deviceState_2).axes[event_0.button as usize] = event_0.value;
                    Input_SetActiveDevice(event_0.device);
                    Input_AppendEvent(event_0);
                }
            }
            1027 => {
                let mut device_4: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Mouse,
                        id: sdl.wheel.which,
                    };
                    init
                };
                Input_EnsureDeviceState(device_4);
                event_0.device = device_4;
                event_0.button = Button_Mouse_ScrollX;
                event_0.value = sdl.wheel.x as f32;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                if event_0.value
                    != Input_GetDeviceValueImpl(event_0.device, event_0.button)
                {
                    Input_SetButton(event_0);
                    Input_AppendEvent(event_0);
                }
                event_0.device = device_4;
                event_0.button = Button_Mouse_ScrollY;
                event_0.value = sdl.wheel.y as f32;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                if event_0.value
                    != Input_GetDeviceValueImpl(event_0.device, event_0.button)
                {
                    Input_SetButton(event_0);
                    Input_AppendEvent(event_0);
                }
            }
            1617 => {
                let mut device_5: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Gamepad,
                        id: sdl.cbutton.which as uint32,
                    };
                    init
                };
                event_0.device = device_5;
                event_0
                    .button = Button_FromSDLControllerButton(
                    sdl.cbutton.button as SDL_GameControllerButton,
                );
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1618 => {
                let mut device_6: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Gamepad,
                        id: sdl.cbutton.which as uint32,
                    };
                    init
                };
                event_0.device = device_6;
                event_0
                    .button = Button_FromSDLControllerButton(
                    sdl.cbutton.button as SDL_GameControllerButton,
                );
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1616 => {
                let mut device_7: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Gamepad,
                        id: sdl.caxis.which as uint32,
                    };
                    init
                };
                let mut value: f32 = Clamp(
                    (sdl.caxis.value as f32 / 32767.0f32) as f64,
                    -1.0f32 as f64,
                    1.0f32 as f64,
                ) as f32;
                let mut axis: SDL_GameControllerAxis = sdl.caxis.axis
                    as SDL_GameControllerAxis;
                if axis as libc::c_int == SDL_CONTROLLER_AXIS_LEFTY as libc::c_int
                    || axis as libc::c_int == SDL_CONTROLLER_AXIS_RIGHTY as libc::c_int
                {
                    value = -value;
                }
                event_0.device = device_7;
                event_0.button = Button_FromSDLControllerAxis(axis);
                event_0.value = value;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            1619 => {
                if SDL_IsGameController(sdl.cdevice.which) as libc::c_uint
                    == SDL_TRUE as libc::c_int as libc::c_uint
                {
                    let mut sdlController: *mut SDL_GameController = SDL_GameControllerOpen(
                        sdl.cdevice.which,
                    );
                    if sdlController.is_null() {
                        Warn(
                            b"Input_Update: SDL_GameControllerOpen failed\0" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        let mut sdlJoystick: *mut SDL_Joystick = SDL_GameControllerGetJoystick(
                            sdlController,
                        );
                        let mut id: uint32 = SDL_JoystickInstanceID(sdlJoystick)
                            as uint32;
                        let mut device_8: Device = {
                            let mut init = Device {
                                type_0: DeviceType_Gamepad,
                                id: id,
                            };
                            init
                        };
                        let mut deviceState_3: *mut DeviceState = Input_EnsureDeviceState(
                            device_8,
                        );
                        (*deviceState_3).isConnected = 1 as libc::c_int != 0;
                    }
                }
            }
            1620 => {
                let mut device_9: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Gamepad,
                        id: sdl.cdevice.which as uint32,
                    };
                    init
                };
                let mut deviceState_4: *mut DeviceState = Input_GetDeviceState(device_9);
                (*deviceState_4).isConnected = 0 as libc::c_int != 0;
                let mut sdlController_0: *mut SDL_GameController = SDL_GameControllerFromInstanceID(
                    sdl.cdevice.which,
                );
                if !sdlController_0.is_null() {
                    SDL_GameControllerClose(sdlController_0);
                }
            }
            1621 => {
                let mut sdlController_1: *mut SDL_GameController = SDL_GameControllerFromInstanceID(
                    sdl.cdevice.which,
                );
                let mut device_10: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Gamepad,
                        id: sdl.cdevice.which as uint32,
                    };
                    init
                };
                let mut deviceState_5: *mut DeviceState = Input_GetDeviceState(
                    device_10,
                );
                let mut iBtn: int32 = Button_Gamepad_Button_First;
                while iBtn <= Button_Gamepad_Button_Last {
                    let mut value_0: f32 = SDL_GameControllerGetButton(
                        sdlController_1,
                        Button_ToSDLControllerButton(iBtn),
                    ) as f32;
                    if value_0 != (*deviceState_5).axes[iBtn as usize] {
                        event_0.device = device_10;
                        event_0.button = iBtn;
                        event_0.value = value_0;
                        event_0
                            .state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(event_0);
                        Input_AppendEvent(event_0);
                    }
                    iBtn += 1;
                }
                let mut iAxis: int32 = Button_Gamepad_Axis_First;
                while iAxis <= Button_Gamepad_Axis_Last {
                    let mut value_1: f32 = SDL_GameControllerGetAxis(
                        sdlController_1,
                        Button_ToSDLControllerAxis(iAxis),
                    ) as f32;
                    value_1 = Clamp(
                        (value_1 / 32767.0f32) as f64,
                        -1.0f32 as f64,
                        1.0f32 as f64,
                    ) as f32;
                    if iAxis == Button_Gamepad_LStickY || iAxis == Button_Gamepad_RStickY
                    {
                        value_1 = -value_1;
                    }
                    if value_1 != (*deviceState_5).axes[iAxis as usize] {
                        event_0.device = device_10;
                        event_0.button = iAxis;
                        event_0.value = value_1;
                        event_0
                            .state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(event_0);
                        Input_AppendEvent(event_0);
                    }
                    iAxis += 1;
                }
            }
            256 => {
                let mut device_11: Device = {
                    let mut init = Device {
                        type_0: DeviceType_Null,
                        id: 0 as libc::c_int as uint32,
                    };
                    init
                };
                event_0.device = device_11;
                event_0.button = Button_System_Exit;
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            512 => {
                if sdl.window.event as libc::c_int
                    == SDL_WINDOWEVENT_FOCUS_GAINED as libc::c_int
                {
                    SDL_CaptureMouse(SDL_TRUE);
                }
                if sdl.window.event as libc::c_int
                    == SDL_WINDOWEVENT_FOCUS_LOST as libc::c_int
                {
                    SDL_CaptureMouse(SDL_FALSE);
                    let mut down_1: *mut InputEvent = this.downButtons_data;
                    let mut __iterend_3: *mut InputEvent = (this.downButtons_data)
                        .offset(this.downButtons_size as isize);
                    while down_1 < __iterend_3 {
                        (*down_1).timestamp = sdl.common.timestamp;
                        (*down_1).value = 0.0f32;
                        (*down_1)
                            .state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(*down_1);
                        Input_AppendEvent(*down_1);
                        down_1 = down_1.offset(1);
                    }
                }
            }
            _ => {}
        }
    }
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Input_LoadGamepadDatabase(mut name: cstr) {
    let mut path: cstr = Resource_GetPath(ResourceType_Other, name);
    let mut result: libc::c_int = SDL_GameControllerAddMappingsFromRW(
        SDL_RWFromFile(path, b"rb\0" as *const u8 as *const libc::c_char),
        1 as libc::c_int,
    );
    if result == -(1 as libc::c_int) {
        Fatal(
            b"Input_Init: Failed to add gamepad mappings\0" as *const u8
                as *const libc::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetPressed(mut button: Button) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: Button_ToDeviceType(button),
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDevicePressedImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDown(mut button: Button) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: Button_ToDeviceType(button),
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDeviceDownImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetReleased(mut button: Button) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: Button_ToDeviceType(button),
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDeviceReleasedImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetValue(mut button: Button) -> f32 {
    let mut device: Device = {
        let mut init = Device {
            type_0: Button_ToDeviceType(button),
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDeviceValueImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetIdleTime() -> f32 {
    return (this.lastTimestamp).wrapping_sub(this.lastEventTimestamp)
        as f32 / 1000.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDevice(mut device: *mut Device) {
    *device = this.activeDevice;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceType() -> DeviceType {
    return this.activeDevice.type_0;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceID() -> uint32 {
    return this.activeDevice.id;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceIdleTime() -> f32 {
    return Input_GetDeviceIdleTimeImpl(this.activeDevice);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDevicePressed(
    mut device: *mut Device,
    mut button: Button,
) -> bool {
    if !Input_GetDeviceExists(*device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDevicePressedImpl(*device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceDown(
    mut device: *mut Device,
    mut button: Button,
) -> bool {
    if !Input_GetDeviceExists(*device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDeviceDownImpl(*device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceReleased(
    mut device: *mut Device,
    mut button: Button,
) -> bool {
    if !Input_GetDeviceExists(*device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDeviceReleasedImpl(*device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceValue(
    mut device: *mut Device,
    mut button: Button,
) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return 0.0f32;
    }
    return Input_GetDeviceValueImpl(*device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceIdleTime(
    mut device: *mut Device,
) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return 3.40282347e+38f32;
    }
    return Input_GetDeviceIdleTimeImpl(*device);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseDelta(mut delta: *mut IVec2) {
    (*delta)
        .x = Input_GetValue(Button_Mouse_X) as libc::c_int - this.lastMousePosition.x;
    (*delta)
        .y = Input_GetValue(Button_Mouse_Y) as libc::c_int - this.lastMousePosition.y;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseIdleTime() -> f32 {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Mouse,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDeviceIdleTimeImpl(device);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetMousePosition(mut position: *mut IVec2) {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Mouse,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    (*position).x = Input_GetDeviceValueImpl(device, Button_Mouse_X) as libc::c_int;
    (*position).y = Input_GetDeviceValueImpl(device, Button_Mouse_Y) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseScroll(mut scroll: *mut IVec2) {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Mouse,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    (*scroll).x = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollX) as libc::c_int;
    (*scroll).y = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollY) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Input_SetMousePosition(mut position: *mut IVec2) {
    SDL_WarpMouseInWindow(0 as *mut SDL_Window, (*position).x, (*position).y);
}
#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisible(mut visible: bool) {
    this.autoHideMouse = 0 as libc::c_int != 0;
    SDL_ShowCursor(
        if visible as libc::c_int != 0 { 1 as libc::c_int } else { 0 as libc::c_int },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisibleAuto() {
    this.autoHideMouse = 1 as libc::c_int != 0;
    Input_SetActiveDevice(this.activeDevice);
}
#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseScroll(mut scroll: *mut IVec2) {
    let mut timestamp: uint32 = SDL_GetTicks();
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Mouse,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    let mut event: InputEvent = {
        let mut init = InputEvent {
            timestamp: 0,
            device: Device { type_0: 0, id: 0 },
            button: 0,
            value: 0.,
            state: 0,
        };
        init
    };
    event.timestamp = timestamp;
    event.device = device;
    event.button = Button_Mouse_ScrollX;
    event.value = (*scroll).x as f32;
    event.state = State_Changed | Input_DetermineButtonState(event);
    if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
        Input_InjectEvent(event);
    }
    event.timestamp = timestamp;
    event.device = device;
    event.button = Button_Mouse_ScrollY;
    event.value = (*scroll).y as f32;
    event.state = State_Changed | Input_DetermineButtonState(event);
    if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
        Input_InjectEvent(event);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardIdleTime() -> f32 {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Keyboard,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    return Input_GetDeviceIdleTimeImpl(device);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardMod(mut modifier: Modifier) -> bool {
    let mut hasMod: bool = 1 as libc::c_int != 0;
    if modifier & Modifier_Alt == Modifier_Alt {
        hasMod = (hasMod as libc::c_int & Input_GetKeyboardAlt() as libc::c_int) != 0;
    }
    if modifier & Modifier_Ctrl == Modifier_Ctrl {
        hasMod = (hasMod as libc::c_int & Input_GetKeyboardCtrl() as libc::c_int)
            != 0;
    }
    if modifier & Modifier_Shift == Modifier_Shift {
        hasMod = (hasMod as libc::c_int & Input_GetKeyboardShift() as libc::c_int)
            != 0;
    }
    return hasMod;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardAlt() -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Keyboard,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (*deviceState).buttons[Button_Keyboard_LAlt as usize] as libc::c_int != 0
        || (*deviceState).buttons[Button_Keyboard_RAlt as usize] as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardCtrl() -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Keyboard,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (*deviceState).buttons[Button_Keyboard_LCtrl as usize] as libc::c_int != 0
        || (*deviceState).buttons[Button_Keyboard_RCtrl as usize] as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardShift() -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Keyboard,
            id: 0 as libc::c_int as uint32,
        };
        init
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    return (*deviceState).buttons[Button_Keyboard_LShift as usize] as libc::c_int != 0
        || (*deviceState).buttons[Button_Keyboard_RShift as usize] as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadIdleTime(mut id: uint32) -> f32 {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Gamepad,
            id: id,
        };
        init
    };
    if !Input_GetDeviceExists(device) {
        return 3.40282347e+38f32;
    }
    return Input_GetDeviceIdleTimeImpl(device);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadPressed(
    mut id: uint32,
    mut button: Button,
) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Gamepad,
            id: id,
        };
        init
    };
    if !Input_GetDeviceExists(device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDevicePressedImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadDown(
    mut id: uint32,
    mut button: Button,
) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Gamepad,
            id: id,
        };
        init
    };
    if !Input_GetDeviceExists(device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDeviceDownImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadReleased(
    mut id: uint32,
    mut button: Button,
) -> bool {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Gamepad,
            id: id,
        };
        init
    };
    if !Input_GetDeviceExists(device) {
        return 0 as libc::c_int != 0;
    }
    return Input_GetDeviceReleasedImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadValue(
    mut id: uint32,
    mut button: Button,
) -> f32 {
    let mut device: Device = {
        let mut init = Device {
            type_0: DeviceType_Gamepad,
            id: id,
        };
        init
    };
    if !Input_GetDeviceExists(device) {
        return 0.0f32;
    }
    return Input_GetDeviceValueImpl(device, button);
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetEventCount() -> int32 {
    return this.events_size;
}
#[no_mangle]
pub unsafe extern "C" fn Input_GetNextEvent(mut event: *mut InputEvent) -> bool {
    if this.events_size == 0 as libc::c_int {
        return 0 as libc::c_int != 0;
    }
    Profiler_Begin(
        (*::core::mem::transmute::<
            &[u8; 19],
            &[libc::c_char; 19],
        >(b"Input_GetNextEvent\0"))
            .as_ptr(),
    );
    *event = *(this.events_data).offset(0);
    if 0 as libc::c_int != this.events_size - 1 as libc::c_int {
        let mut curr: *mut libc::c_void = (this.events_data)
            .offset(0)
            .offset(0) as *mut libc::c_void;
        let mut next: *mut libc::c_void = (this.events_data)
            .offset(0)
            .offset(1) as *mut libc::c_void;
        let mut elemSize: usize = ::core::mem::size_of::<InputEvent>() as usize;
        MemMove(
            curr,
            next,
            ((this.events_size - 1 as libc::c_int - 0 as libc::c_int) as usize).wrapping_mul(elemSize),
        );
    }
    this.events_size -= 1;
    Profiler_End();
    return 1 as libc::c_int != 0;
}
