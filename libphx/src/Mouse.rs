use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
extern "C" {
    pub type SDL_Window;
    fn SDL_WarpMouseInWindow(window: *mut SDL_Window, x: i32, y: i32);
    fn SDL_GetMouseState(x: *mut i32, y: *mut i32) -> u32;
    fn SDL_GetPerformanceFrequency() -> u64;
    fn SDL_GetPerformanceCounter() -> u64;
    fn SDL_ShowCursor(toggle: i32) -> i32;
    fn SDL_GetGlobalMouseState(x: *mut i32, y: *mut i32) -> u32;
}

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
    scrollAmount = 0 as i32;
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
    scrollAmount = 0 as i32;
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
    return now.wrapping_sub(lastAction) as f64
        / SDL_GetPerformanceFrequency() as f64;
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
    return scrollAmount;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_SetPosition(mut x: i32, mut y: i32) {
    SDL_WarpMouseInWindow(0 as *mut SDL_Window, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_SetVisible(mut visible: bool) {
    SDL_ShowCursor(
        if visible as i32 != 0 { 1 as i32 } else { 0 as i32 },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Down(mut button: MouseButton) -> bool {
    button = (1 as i32) << button - 1 as i32;
    return SDL_GetMouseState(0 as *mut i32, 0 as *mut i32)
        & button as u32 > 0 as i32 as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Pressed(mut button: MouseButton) -> bool {
    button = (1 as i32) << button - 1 as i32;
    let mut current: u32 = SDL_GetMouseState(
        0 as *mut i32,
        0 as *mut i32,
    );
    return current & button as u32 != 0
        && lastState & button as u32 == 0;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Released(mut button: MouseButton) -> bool {
    button = (1 as i32) << button - 1 as i32;
    let mut current: u32 = SDL_GetMouseState(
        0 as *mut i32,
        0 as *mut i32,
    );
    return current & button as u32 == 0
        && lastState & button as u32 != 0;
}
