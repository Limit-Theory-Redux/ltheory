use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

pub type uchar = libc::c_uchar;
pub type Key = uchar;

#[no_mangle]
pub static mut Key_A: Key = SDL_Scancode::SDL_SCANCODE_A as Key;

#[no_mangle]
pub static mut Key_B: Key = SDL_Scancode::SDL_SCANCODE_B as Key;

#[no_mangle]
pub static mut Key_C: Key = SDL_Scancode::SDL_SCANCODE_C as Key;

#[no_mangle]
pub static mut Key_D: Key = SDL_Scancode::SDL_SCANCODE_D as Key;

#[no_mangle]
pub static mut Key_E: Key = SDL_Scancode::SDL_SCANCODE_E as Key;

#[no_mangle]
pub static mut Key_F: Key = SDL_Scancode::SDL_SCANCODE_F as Key;

#[no_mangle]
pub static mut Key_G: Key = SDL_Scancode::SDL_SCANCODE_G as Key;

#[no_mangle]
pub static mut Key_H: Key = SDL_Scancode::SDL_SCANCODE_H as Key;

#[no_mangle]
pub static mut Key_I: Key = SDL_Scancode::SDL_SCANCODE_I as Key;

#[no_mangle]
pub static mut Key_J: Key = SDL_Scancode::SDL_SCANCODE_J as Key;

#[no_mangle]
pub static mut Key_K: Key = SDL_Scancode::SDL_SCANCODE_K as Key;

#[no_mangle]
pub static mut Key_L: Key = SDL_Scancode::SDL_SCANCODE_L as Key;

#[no_mangle]
pub static mut Key_M: Key = SDL_Scancode::SDL_SCANCODE_M as Key;

#[no_mangle]
pub static mut Key_N: Key = SDL_Scancode::SDL_SCANCODE_N as Key;

#[no_mangle]
pub static mut Key_O: Key = SDL_Scancode::SDL_SCANCODE_O as Key;

#[no_mangle]
pub static mut Key_P: Key = SDL_Scancode::SDL_SCANCODE_P as Key;

#[no_mangle]
pub static mut Key_Q: Key = SDL_Scancode::SDL_SCANCODE_Q as Key;

#[no_mangle]
pub static mut Key_R: Key = SDL_Scancode::SDL_SCANCODE_R as Key;

#[no_mangle]
pub static mut Key_S: Key = SDL_Scancode::SDL_SCANCODE_S as Key;

#[no_mangle]
pub static mut Key_T: Key = SDL_Scancode::SDL_SCANCODE_T as Key;

#[no_mangle]
pub static mut Key_U: Key = SDL_Scancode::SDL_SCANCODE_U as Key;

#[no_mangle]
pub static mut Key_V: Key = SDL_Scancode::SDL_SCANCODE_V as Key;

#[no_mangle]
pub static mut Key_W: Key = SDL_Scancode::SDL_SCANCODE_W as Key;

#[no_mangle]
pub static mut Key_X: Key = SDL_Scancode::SDL_SCANCODE_X as Key;

#[no_mangle]
pub static mut Key_Y: Key = SDL_Scancode::SDL_SCANCODE_Y as Key;

#[no_mangle]
pub static mut Key_Z: Key = SDL_Scancode::SDL_SCANCODE_Z as Key;

#[no_mangle]
pub static mut Key_N0: Key = SDL_Scancode::SDL_SCANCODE_0 as Key;

#[no_mangle]
pub static mut Key_N1: Key = SDL_Scancode::SDL_SCANCODE_1 as Key;

#[no_mangle]
pub static mut Key_N2: Key = SDL_Scancode::SDL_SCANCODE_2 as Key;

#[no_mangle]
pub static mut Key_N3: Key = SDL_Scancode::SDL_SCANCODE_3 as Key;

#[no_mangle]
pub static mut Key_N4: Key = SDL_Scancode::SDL_SCANCODE_4 as Key;

#[no_mangle]
pub static mut Key_N5: Key = SDL_Scancode::SDL_SCANCODE_5 as Key;

#[no_mangle]
pub static mut Key_N6: Key = SDL_Scancode::SDL_SCANCODE_6 as Key;

#[no_mangle]
pub static mut Key_N7: Key = SDL_Scancode::SDL_SCANCODE_7 as Key;

#[no_mangle]
pub static mut Key_N8: Key = SDL_Scancode::SDL_SCANCODE_8 as Key;

#[no_mangle]
pub static mut Key_N9: Key = SDL_Scancode::SDL_SCANCODE_9 as Key;

#[no_mangle]
pub static mut Key_F1: Key = SDL_Scancode::SDL_SCANCODE_F1 as Key;

#[no_mangle]
pub static mut Key_F2: Key = SDL_Scancode::SDL_SCANCODE_F2 as Key;

#[no_mangle]
pub static mut Key_F3: Key = SDL_Scancode::SDL_SCANCODE_F3 as Key;

#[no_mangle]
pub static mut Key_F4: Key = SDL_Scancode::SDL_SCANCODE_F4 as Key;

#[no_mangle]
pub static mut Key_F5: Key = SDL_Scancode::SDL_SCANCODE_F5 as Key;

#[no_mangle]
pub static mut Key_F6: Key = SDL_Scancode::SDL_SCANCODE_F6 as Key;

#[no_mangle]
pub static mut Key_F7: Key = SDL_Scancode::SDL_SCANCODE_F7 as Key;

#[no_mangle]
pub static mut Key_F8: Key = SDL_Scancode::SDL_SCANCODE_F8 as Key;

#[no_mangle]
pub static mut Key_F9: Key = SDL_Scancode::SDL_SCANCODE_F9 as Key;

#[no_mangle]
pub static mut Key_F10: Key = SDL_Scancode::SDL_SCANCODE_F10 as Key;

#[no_mangle]
pub static mut Key_F11: Key = SDL_Scancode::SDL_SCANCODE_F11 as Key;

#[no_mangle]
pub static mut Key_F12: Key = SDL_Scancode::SDL_SCANCODE_F12 as Key;

#[no_mangle]
pub static mut Key_F13: Key = SDL_Scancode::SDL_SCANCODE_F13 as Key;

#[no_mangle]
pub static mut Key_F14: Key = SDL_Scancode::SDL_SCANCODE_F14 as Key;

#[no_mangle]
pub static mut Key_F15: Key = SDL_Scancode::SDL_SCANCODE_F15 as Key;

#[no_mangle]
pub static mut Key_F16: Key = SDL_Scancode::SDL_SCANCODE_F16 as Key;

#[no_mangle]
pub static mut Key_F17: Key = SDL_Scancode::SDL_SCANCODE_F17 as Key;

#[no_mangle]
pub static mut Key_F18: Key = SDL_Scancode::SDL_SCANCODE_F18 as Key;

#[no_mangle]
pub static mut Key_F19: Key = SDL_Scancode::SDL_SCANCODE_F19 as Key;

#[no_mangle]
pub static mut Key_F20: Key = SDL_Scancode::SDL_SCANCODE_F20 as Key;

#[no_mangle]
pub static mut Key_F21: Key = SDL_Scancode::SDL_SCANCODE_F21 as Key;

#[no_mangle]
pub static mut Key_F22: Key = SDL_Scancode::SDL_SCANCODE_F22 as Key;

#[no_mangle]
pub static mut Key_F23: Key = SDL_Scancode::SDL_SCANCODE_F23 as Key;

#[no_mangle]
pub static mut Key_F24: Key = SDL_Scancode::SDL_SCANCODE_F24 as Key;

#[no_mangle]
pub static mut Key_KP0: Key = SDL_Scancode::SDL_SCANCODE_KP_0 as Key;

#[no_mangle]
pub static mut Key_KP1: Key = SDL_Scancode::SDL_SCANCODE_KP_1 as Key;

#[no_mangle]
pub static mut Key_KP2: Key = SDL_Scancode::SDL_SCANCODE_KP_2 as Key;

#[no_mangle]
pub static mut Key_KP3: Key = SDL_Scancode::SDL_SCANCODE_KP_3 as Key;

#[no_mangle]
pub static mut Key_KP4: Key = SDL_Scancode::SDL_SCANCODE_KP_4 as Key;

#[no_mangle]
pub static mut Key_KP5: Key = SDL_Scancode::SDL_SCANCODE_KP_5 as Key;

#[no_mangle]
pub static mut Key_KP6: Key = SDL_Scancode::SDL_SCANCODE_KP_6 as Key;

#[no_mangle]
pub static mut Key_KP7: Key = SDL_Scancode::SDL_SCANCODE_KP_7 as Key;

#[no_mangle]
pub static mut Key_KP8: Key = SDL_Scancode::SDL_SCANCODE_KP_8 as Key;

#[no_mangle]
pub static mut Key_KP9: Key = SDL_Scancode::SDL_SCANCODE_KP_9 as Key;

#[no_mangle]
pub static mut Key_KPNumLock: Key = SDL_Scancode::SDL_SCANCODE_NUMLOCKCLEAR as Key;

#[no_mangle]
pub static mut Key_KPDivide: Key = SDL_Scancode::SDL_SCANCODE_KP_DIVIDE as Key;

#[no_mangle]
pub static mut Key_KPMultiply: Key = SDL_Scancode::SDL_SCANCODE_KP_MULTIPLY as Key;

#[no_mangle]
pub static mut Key_KPSubtract: Key = SDL_Scancode::SDL_SCANCODE_KP_MINUS as Key;

#[no_mangle]
pub static mut Key_KPAdd: Key = SDL_Scancode::SDL_SCANCODE_KP_PLUS as Key;

#[no_mangle]
pub static mut Key_KPEnter: Key = SDL_Scancode::SDL_SCANCODE_KP_ENTER as Key;

#[no_mangle]
pub static mut Key_KPDecimal: Key = SDL_Scancode::SDL_SCANCODE_KP_DECIMAL as Key;

#[no_mangle]
pub static mut Key_Backspace: Key = SDL_Scancode::SDL_SCANCODE_BACKSPACE as Key;

#[no_mangle]
pub static mut Key_Escape: Key = SDL_Scancode::SDL_SCANCODE_ESCAPE as Key;

#[no_mangle]
pub static mut Key_Return: Key = SDL_Scancode::SDL_SCANCODE_RETURN as Key;

#[no_mangle]
pub static mut Key_Space: Key = SDL_Scancode::SDL_SCANCODE_SPACE as Key;

#[no_mangle]
pub static mut Key_Tab: Key = SDL_Scancode::SDL_SCANCODE_TAB as Key;

#[no_mangle]
pub static mut Key_Backtick: Key = SDL_Scancode::SDL_SCANCODE_GRAVE as Key;

#[no_mangle]
pub static mut Key_CapsLock: Key = SDL_Scancode::SDL_SCANCODE_CAPSLOCK as Key;

#[no_mangle]
pub static mut Key_Minus: Key = SDL_Scancode::SDL_SCANCODE_MINUS as Key;

#[no_mangle]
pub static mut Key_Equals: Key = SDL_Scancode::SDL_SCANCODE_EQUALS as Key;

#[no_mangle]
pub static mut Key_LBracket: Key = SDL_Scancode::SDL_SCANCODE_LEFTBRACKET as Key;

#[no_mangle]
pub static mut Key_RBracket: Key = SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET as Key;

#[no_mangle]
pub static mut Key_Backslash: Key = SDL_Scancode::SDL_SCANCODE_BACKSLASH as Key;

#[no_mangle]
pub static mut Key_Semicolon: Key = SDL_Scancode::SDL_SCANCODE_SEMICOLON as Key;

#[no_mangle]
pub static mut Key_Apostrophe: Key = SDL_Scancode::SDL_SCANCODE_APOSTROPHE as Key;

#[no_mangle]
pub static mut Key_Comma: Key = SDL_Scancode::SDL_SCANCODE_COMMA as Key;

#[no_mangle]
pub static mut Key_Period: Key = SDL_Scancode::SDL_SCANCODE_PERIOD as Key;

#[no_mangle]
pub static mut Key_Slash: Key = SDL_Scancode::SDL_SCANCODE_SLASH as Key;

#[no_mangle]
pub static mut Key_PrintScreen: Key = SDL_Scancode::SDL_SCANCODE_PRINTSCREEN as Key;

#[no_mangle]
pub static mut Key_ScrollLock: Key = SDL_Scancode::SDL_SCANCODE_SCROLLLOCK as Key;

#[no_mangle]
pub static mut Key_Pause: Key = SDL_Scancode::SDL_SCANCODE_PAUSE as Key;

#[no_mangle]
pub static mut Key_Insert: Key = SDL_Scancode::SDL_SCANCODE_INSERT as Key;

#[no_mangle]
pub static mut Key_Home: Key = SDL_Scancode::SDL_SCANCODE_HOME as Key;

#[no_mangle]
pub static mut Key_PageUp: Key = SDL_Scancode::SDL_SCANCODE_PAGEUP as Key;

#[no_mangle]
pub static mut Key_PageDown: Key = SDL_Scancode::SDL_SCANCODE_PAGEDOWN as Key;

#[no_mangle]
pub static mut Key_Delete: Key = SDL_Scancode::SDL_SCANCODE_DELETE as Key;

#[no_mangle]
pub static mut Key_Right: Key = SDL_Scancode::SDL_SCANCODE_RIGHT as Key;

#[no_mangle]
pub static mut Key_Left: Key = SDL_Scancode::SDL_SCANCODE_LEFT as Key;

#[no_mangle]
pub static mut Key_Down: Key = SDL_Scancode::SDL_SCANCODE_DOWN as Key;

#[no_mangle]
pub static mut Key_Up: Key = SDL_Scancode::SDL_SCANCODE_UP as Key;

#[no_mangle]
pub static mut Key_LCtrl: Key = SDL_Scancode::SDL_SCANCODE_LCTRL as Key;

#[no_mangle]
pub static mut Key_LShift: Key = SDL_Scancode::SDL_SCANCODE_LSHIFT as Key;

#[no_mangle]
pub static mut Key_LAlt: Key = SDL_Scancode::SDL_SCANCODE_LALT as Key;

#[no_mangle]
pub static mut Key_LMeta: Key = SDL_Scancode::SDL_SCANCODE_LGUI as Key;

#[no_mangle]
pub static mut Key_RCtrl: Key = SDL_Scancode::SDL_SCANCODE_RCTRL as Key;

#[no_mangle]
pub static mut Key_RShift: Key = SDL_Scancode::SDL_SCANCODE_RSHIFT as Key;

#[no_mangle]
pub static mut Key_RAlt: Key = SDL_Scancode::SDL_SCANCODE_RALT as Key;

#[no_mangle]
pub static mut Key_RMeta: Key = SDL_Scancode::SDL_SCANCODE_RGUI as Key;
