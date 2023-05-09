use crate::internal::Memory::*;

use crate::Key::*;

use sdl2_sys::*;

static mut lastAction: u64 = 0;

static mut stateLast: *mut libc::c_uchar = std::ptr::null_mut();

static mut stateCurr: *mut libc::c_uchar = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Init() {
    let mut size: i32 = 0;
    let state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
    stateLast = MemAlloc((std::mem::size_of::<libc::c_uchar>()).wrapping_mul(size as usize))
        as *mut libc::c_uchar;
    stateCurr = MemAlloc((std::mem::size_of::<libc::c_uchar>()).wrapping_mul(size as usize))
        as *mut libc::c_uchar;
    MemCpy(stateLast as *mut _, state as *const _, size as usize);
    MemCpy(stateCurr as *mut _, state as *const _, size as usize);
    lastAction = SDL_GetPerformanceCounter();
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Free() {
    MemFree(stateLast as *const _);
    MemFree(stateCurr as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_UpdatePre() {
    let mut size: i32 = 0;
    let state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
    MemCpy(stateLast as *mut _, state as *const _, size as usize);
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_UpdatePost() {
    let mut size: i32 = 0;
    let state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
    MemCpy(stateCurr as *mut _, state as *const _, size as usize);

    for i in 0..size {
        if *stateCurr.offset(i as isize) as i32 != *stateLast.offset(i as isize) as i32 {
            lastAction = SDL_GetPerformanceCounter();
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Down(key: Key) -> bool {
    *stateCurr.offset(key as isize) as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Pressed(key: Key) -> bool {
    *stateCurr.offset(key as isize) as i32 != 0 && *stateLast.offset(key as isize) == 0
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Released(key: Key) -> bool {
    *stateCurr.offset(key as isize) == 0 && *stateLast.offset(key as isize) as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_GetIdleTime() -> f64 {
    let now: u64 = SDL_GetPerformanceCounter();
    now.wrapping_sub(lastAction) as f64 / SDL_GetPerformanceFrequency() as f64
}

#[no_mangle]
pub unsafe extern "C" fn KeyMod_Alt() -> bool {
    *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LALT as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RALT as isize) as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn KeyMod_Ctrl() -> bool {
    *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LCTRL as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RCTRL as isize) as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn KeyMod_Shift() -> bool {
    *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LSHIFT as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RSHIFT as isize) as i32 != 0
}
