use sdl2_sys::*;

pub type WindowMode = u32;

#[no_mangle]
pub static WindowMode_AlwaysOnTop: WindowMode =
    SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as WindowMode;

#[no_mangle]
pub static WindowMode_Borderless: WindowMode = SDL_WindowFlags::SDL_WINDOW_BORDERLESS as WindowMode;

#[no_mangle]
pub static WindowMode_Fullscreen: WindowMode =
    SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as WindowMode;

#[no_mangle]
pub static WindowMode_Hidden: WindowMode = SDL_WindowFlags::SDL_WINDOW_HIDDEN as WindowMode;

#[no_mangle]
pub static WindowMode_Maximized: WindowMode = SDL_WindowFlags::SDL_WINDOW_MAXIMIZED as WindowMode;

#[no_mangle]
pub static WindowMode_Minimized: WindowMode = SDL_WindowFlags::SDL_WINDOW_MINIMIZED as WindowMode;

#[no_mangle]
pub static WindowMode_Resizable: WindowMode = SDL_WindowFlags::SDL_WINDOW_RESIZABLE as WindowMode;

#[no_mangle]
pub static WindowMode_Shown: WindowMode = SDL_WindowFlags::SDL_WINDOW_SHOWN as WindowMode;
