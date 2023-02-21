use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
extern "C" {
    pub type SDL_Window;
    fn SDL_WarpMouseInWindow(window: *mut SDL_Window, x: libc::c_int, y: libc::c_int);
    fn SDL_GetMouseState(x: *mut libc::c_int, y: *mut libc::c_int) -> u32;
    fn SDL_GetPerformanceFrequency() -> u64;
    fn SDL_GetPerformanceCounter() -> u64;
    fn SDL_ShowCursor(toggle: libc::c_int) -> libc::c_int;
    fn SDL_GetGlobalMouseState(x: *mut libc::c_int, y: *mut libc::c_int) -> u32;
}

pub type MouseButton = i32;
#[no_mangle]
pub static mut lastX: libc::c_int = 0;
#[no_mangle]
pub static mut lastY: libc::c_int = 0;
#[no_mangle]
pub static mut lastState: u32 = 0;
static mut lastAction: u64 = 0;
static mut scrollAmount: libc::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn Mouse_Init() {
    lastState = SDL_GetMouseState(&mut lastX, &mut lastY);
    lastAction = SDL_GetPerformanceCounter();
    scrollAmount = 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Free() {}
#[no_mangle]
pub unsafe extern "C" fn Mouse_SetScroll(mut amount: libc::c_int) {
    scrollAmount = amount;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Update() {
    let mut lx: libc::c_int = lastX;
    let mut ly: libc::c_int = lastY;
    let mut state: u32 = lastState;
    lastState = SDL_GetMouseState(&mut lastX, &mut lastY);
    if lx != lastX || ly != lastY || state != lastState {
        lastAction = SDL_GetPerformanceCounter();
    }
    scrollAmount = 0 as libc::c_int;
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
pub unsafe extern "C" fn Mouse_GetScroll() -> libc::c_int {
    return scrollAmount;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_SetPosition(mut x: libc::c_int, mut y: libc::c_int) {
    SDL_WarpMouseInWindow(0 as *mut SDL_Window, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_SetVisible(mut visible: bool) {
    SDL_ShowCursor(
        if visible as libc::c_int != 0 { 1 as libc::c_int } else { 0 as libc::c_int },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Down(mut button: MouseButton) -> bool {
    button = (1 as libc::c_int) << button - 1 as libc::c_int;
    return SDL_GetMouseState(0 as *mut libc::c_int, 0 as *mut libc::c_int)
        & button as libc::c_uint > 0 as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Pressed(mut button: MouseButton) -> bool {
    button = (1 as libc::c_int) << button - 1 as libc::c_int;
    let mut current: u32 = SDL_GetMouseState(
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    );
    return current & button as libc::c_uint != 0
        && lastState & button as libc::c_uint == 0;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Released(mut button: MouseButton) -> bool {
    button = (1 as libc::c_int) << button - 1 as libc::c_int;
    let mut current: u32 = SDL_GetMouseState(
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    );
    return current & button as libc::c_uint == 0
        && lastState & button as libc::c_uint != 0;
}
