use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
extern "C" {
    pub type SDL_Window;
    fn SDL_WarpMouseInWindow(window: *mut SDL_Window, x: libc::c_int, y: libc::c_int);
    fn SDL_GetMouseState(x: *mut libc::c_int, y: *mut libc::c_int) -> Uint32;
    fn SDL_GetPerformanceFrequency() -> Uint64;
    fn SDL_GetPerformanceCounter() -> Uint64;
    fn SDL_ShowCursor(toggle: libc::c_int) -> libc::c_int;
    fn SDL_GetGlobalMouseState(x: *mut libc::c_int, y: *mut libc::c_int) -> Uint32;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;

pub type MouseButton = int32;
pub type Uint32 = uint32_t;
pub type Uint64 = uint64_t;
#[no_mangle]
pub static mut lastX: libc::c_int = 0;
#[no_mangle]
pub static mut lastY: libc::c_int = 0;
#[no_mangle]
pub static mut lastState: uint32 = 0;
static mut lastAction: uint64 = 0;
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
    let mut state: uint32 = lastState;
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
pub unsafe extern "C" fn Mouse_GetIdleTime() -> libc::c_double {
    let mut now: uint64 = SDL_GetPerformanceCounter();
    return now.wrapping_sub(lastAction) as libc::c_double
        / SDL_GetPerformanceFrequency() as libc::c_double;
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
    let mut current: uint32 = SDL_GetMouseState(
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    );
    return current & button as libc::c_uint != 0
        && lastState & button as libc::c_uint == 0;
}
#[no_mangle]
pub unsafe extern "C" fn Mouse_Released(mut button: MouseButton) -> bool {
    button = (1 as libc::c_int) << button - 1 as libc::c_int;
    let mut current: uint32 = SDL_GetMouseState(
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    );
    return current & button as libc::c_uint == 0
        && lastState & button as libc::c_uint != 0;
}
