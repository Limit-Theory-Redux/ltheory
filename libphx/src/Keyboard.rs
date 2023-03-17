use crate::internal::Memory::*;
use crate::Key::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

static mut lastAction: u64 = 0;
static mut stateLast: *mut libc::c_uchar = std::ptr::null_mut();
static mut stateCurr: *mut libc::c_uchar = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn Keyboard_Init() {
    let mut size: i32 = 0;
    let mut state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
    stateLast =
        MemAlloc((::core::mem::size_of::<libc::c_uchar>()).wrapping_mul(size as usize)) as *mut libc::c_uchar;
    stateCurr =
        MemAlloc((::core::mem::size_of::<libc::c_uchar>()).wrapping_mul(size as usize)) as *mut libc::c_uchar;
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
    let mut state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
    MemCpy(
        stateLast as *mut libc::c_void,
        state as *const libc::c_void,
        size as usize,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Keyboard_UpdatePost() {
    let mut size: i32 = 0;
    let mut state: *const libc::c_uchar = SDL_GetKeyboardState(&mut size);
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
    return *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LALT as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RALT as i32 as isize) as i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn KeyMod_Ctrl() -> bool {
    return *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LCTRL as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RCTRL as i32 as isize) as i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn KeyMod_Shift() -> bool {
    return *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_LSHIFT as i32 as isize) as i32 != 0
        || *stateCurr.offset(SDL_Scancode::SDL_SCANCODE_RSHIFT as i32 as isize) as i32 != 0;
}
