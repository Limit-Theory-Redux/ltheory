use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type uint32_t = libc::c_uint;
pub type C2RustUnnamed = libc::c_uint;
pub const SDL_WINDOW_INPUT_GRABBED: C2RustUnnamed = 256;
pub const SDL_WINDOW_METAL: C2RustUnnamed = 536870912;
pub const SDL_WINDOW_VULKAN: C2RustUnnamed = 268435456;
pub const SDL_WINDOW_KEYBOARD_GRABBED: C2RustUnnamed = 1048576;
pub const SDL_WINDOW_POPUP_MENU: C2RustUnnamed = 524288;
pub const SDL_WINDOW_TOOLTIP: C2RustUnnamed = 262144;
pub const SDL_WINDOW_UTILITY: C2RustUnnamed = 131072;
pub const SDL_WINDOW_SKIP_TASKBAR: C2RustUnnamed = 65536;
pub const SDL_WINDOW_ALWAYS_ON_TOP: C2RustUnnamed = 32768;
pub const SDL_WINDOW_MOUSE_CAPTURE: C2RustUnnamed = 16384;
pub const SDL_WINDOW_ALLOW_HIGHDPI: C2RustUnnamed = 8192;
pub const SDL_WINDOW_FOREIGN: C2RustUnnamed = 2048;
pub const SDL_WINDOW_FULLSCREEN_DESKTOP: C2RustUnnamed = 4097;
pub const SDL_WINDOW_MOUSE_FOCUS: C2RustUnnamed = 1024;
pub const SDL_WINDOW_INPUT_FOCUS: C2RustUnnamed = 512;
pub const SDL_WINDOW_MOUSE_GRABBED: C2RustUnnamed = 256;
pub const SDL_WINDOW_MAXIMIZED: C2RustUnnamed = 128;
pub const SDL_WINDOW_MINIMIZED: C2RustUnnamed = 64;
pub const SDL_WINDOW_RESIZABLE: C2RustUnnamed = 32;
pub const SDL_WINDOW_BORDERLESS: C2RustUnnamed = 16;
pub const SDL_WINDOW_HIDDEN: C2RustUnnamed = 8;
pub const SDL_WINDOW_SHOWN: C2RustUnnamed = 4;
pub const SDL_WINDOW_OPENGL: C2RustUnnamed = 2;
pub const SDL_WINDOW_FULLSCREEN: C2RustUnnamed = 1;
pub type uint32 = uint32_t;
pub type WindowMode = uint32;
#[no_mangle]
pub static mut WindowMode_AlwaysOnTop: WindowMode = SDL_WINDOW_ALWAYS_ON_TOP
    as libc::c_int as WindowMode;
#[no_mangle]
pub static mut WindowMode_Borderless: WindowMode = SDL_WINDOW_BORDERLESS as libc::c_int
    as WindowMode;
#[no_mangle]
pub static mut WindowMode_Fullscreen: WindowMode = SDL_WINDOW_FULLSCREEN_DESKTOP
    as libc::c_int as WindowMode;
#[no_mangle]
pub static mut WindowMode_Hidden: WindowMode = SDL_WINDOW_HIDDEN as libc::c_int
    as WindowMode;
#[no_mangle]
pub static mut WindowMode_Maximized: WindowMode = SDL_WINDOW_MAXIMIZED as libc::c_int
    as WindowMode;
#[no_mangle]
pub static mut WindowMode_Minimized: WindowMode = SDL_WINDOW_MINIMIZED as libc::c_int
    as WindowMode;
#[no_mangle]
pub static mut WindowMode_Resizable: WindowMode = SDL_WINDOW_RESIZABLE as libc::c_int
    as WindowMode;
#[no_mangle]
pub static mut WindowMode_Shown: WindowMode = SDL_WINDOW_SHOWN as libc::c_int
    as WindowMode;
