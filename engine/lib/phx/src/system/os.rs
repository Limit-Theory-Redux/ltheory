use std::ffi::CStr;

use crate::common::*;

use sdl2_sys::*;

#[no_mangle]
pub unsafe extern "C" fn OS_GetClipboard() -> *const libc::c_char {
    SDL_GetClipboardText() as *const libc::c_char
}

#[no_mangle]
pub unsafe extern "C" fn OS_GetCPUCount() -> i32 {
    SDL_GetCPUCount()
}

#[no_mangle]
pub unsafe extern "C" fn OS_GetVideoDriver() -> *const libc::c_char {
    SDL_GetCurrentVideoDriver()
}

#[no_mangle]
pub unsafe extern "C" fn OS_SetClipboard(text: *const libc::c_char) {
    if SDL_SetClipboardText(text) != 0 {
        Fatal!("OS_SetClipboard: {:?}", CStr::from_ptr(SDL_GetError()));
    }
}
