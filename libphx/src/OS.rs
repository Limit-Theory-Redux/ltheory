use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn SDL_GetClipboardText() -> *mut libc::c_char;
    fn SDL_GetCPUCount() -> i32;
    fn SDL_SetClipboardText(text: *const libc::c_char) -> i32;
    fn SDL_GetError() -> *const libc::c_char;
    fn SDL_GetCurrentVideoDriver() -> *const libc::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn OS_GetClipboard() -> *const libc::c_char {
    return SDL_GetClipboardText() as *const libc::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn OS_GetCPUCount() -> i32 {
    return SDL_GetCPUCount();
}
#[no_mangle]
pub unsafe extern "C" fn OS_GetVideoDriver() -> *const libc::c_char {
    return SDL_GetCurrentVideoDriver();
}
#[no_mangle]
pub unsafe extern "C" fn OS_SetClipboard(mut text: *const libc::c_char) {
    if SDL_SetClipboardText(text) != 0 as i32 {
        Fatal(
            b"OS_SetClipboard: %s\0" as *const u8 as *const libc::c_char,
            SDL_GetError(),
        );
    }
}
