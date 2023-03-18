use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use libc;
use sdl2_sys::*;

pub type MouseButton = i32;

#[no_mangle]
pub static mut lastX: i32 = 0;

#[no_mangle]
pub static mut lastY: i32 = 0;

#[no_mangle]
pub static mut lastState: u32 = 0;

static mut lastAction: u64 = 0;

static mut scrollAmount: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn Mouse_Init() {
    lastState = SDL_GetMouseState(&mut lastX, &mut lastY);
    lastAction = SDL_GetPerformanceCounter();
    scrollAmount = 0_i32;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Free() {}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetScroll(mut amount: i32) {
    scrollAmount = amount;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Update() {
    let mut lx: i32 = lastX;
    let mut ly: i32 = lastY;
    let mut state: u32 = lastState;
    lastState = SDL_GetMouseState(&mut lastX, &mut lastY);
    if lx != lastX || ly != lastY || state != lastState {
        lastAction = SDL_GetPerformanceCounter();
    }
    scrollAmount = 0_i32;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetDelta(mut out: *mut IVec2) {
    SDL_GetMouseState(&mut (*out).x, &mut (*out).y);
    (*out).x -= lastX;
    (*out).y -= lastY;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetIdleTime() -> f64 {
    let mut now: u64 = SDL_GetPerformanceCounter();
    now.wrapping_sub(lastAction) as f64 / SDL_GetPerformanceFrequency() as f64
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetPosition(mut out: *mut IVec2) {
    SDL_GetMouseState(&mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetPositionGlobal(mut out: *mut IVec2) {
    SDL_GetGlobalMouseState(&mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetScroll() -> i32 {
    scrollAmount
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetPosition(mut x: i32, mut y: i32) {
    SDL_WarpMouseInWindow(std::ptr::null_mut(), x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetVisible(mut visible: bool) {
    SDL_ShowCursor(if visible as i32 != 0 { 1_i32 } else { 0_i32 });
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Down(mut button: MouseButton) -> bool {
    button = 1_i32 << button - 1_i32;
    SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut()) & button as u32 > 0_u32
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Pressed(mut button: MouseButton) -> bool {
    button = 1_i32 << button - 1_i32;
    let mut current: u32 = SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut());
    current & button as u32 != 0 && lastState & button as u32 == 0
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Released(mut button: MouseButton) -> bool {
    button = 1_i32 << button - 1_i32;
    let mut current: u32 = SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut());
    current & button as u32 == 0 && lastState & button as u32 != 0
}
