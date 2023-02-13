use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
    fn SDL_GetClipboardText() -> *mut libc::c_char;
    fn SDL_GetCPUCount() -> libc::c_int;
    fn SDL_SetClipboardText(text: *const libc::c_char) -> libc::c_int;
    fn SDL_GetError() -> *const libc::c_char;
    fn SDL_GetCurrentVideoDriver() -> *const libc::c_char;
}
pub type cstr = *const libc::c_char;
#[no_mangle]
pub unsafe extern "C" fn OS_GetClipboard() -> cstr {
    return SDL_GetClipboardText() as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn OS_GetCPUCount() -> libc::c_int {
    return SDL_GetCPUCount();
}
#[no_mangle]
pub unsafe extern "C" fn OS_GetVideoDriver() -> cstr {
    return SDL_GetCurrentVideoDriver();
}
#[no_mangle]
pub unsafe extern "C" fn OS_SetClipboard(mut text: cstr) {
    if SDL_SetClipboardText(text) != 0 as libc::c_int {
        Fatal(
            b"OS_SetClipboard: %s\0" as *const u8 as *const libc::c_char,
            SDL_GetError(),
        );
    }
}
