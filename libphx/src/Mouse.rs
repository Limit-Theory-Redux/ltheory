use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use crate::MouseButton::*;
use libc;
use sdl2_sys::*;

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
    scrollAmount = 0;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Free() {}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetScroll(amount: i32) {
    scrollAmount = amount;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Update() {
    let lx: i32 = lastX;
    let ly: i32 = lastY;
    let state: u32 = lastState;
    lastState = SDL_GetMouseState(&mut lastX, &mut lastY);
    if lx != lastX || ly != lastY || state != lastState {
        lastAction = SDL_GetPerformanceCounter();
    }
    scrollAmount = 0;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetDelta(out: *mut IVec2) {
    SDL_GetMouseState(&mut (*out).x, &mut (*out).y);
    (*out).x -= lastX;
    (*out).y -= lastY;
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetIdleTime() -> f64 {
    let now: u64 = SDL_GetPerformanceCounter();
    now.wrapping_sub(lastAction) as f64 / SDL_GetPerformanceFrequency() as f64
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetPosition(out: *mut IVec2) {
    SDL_GetMouseState(&mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetPositionGlobal(out: *mut IVec2) {
    SDL_GetGlobalMouseState(&mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_GetScroll() -> i32 {
    scrollAmount
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetPosition(x: i32, y: i32) {
    SDL_WarpMouseInWindow(std::ptr::null_mut(), x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_SetVisible(visible: bool) {
    SDL_ShowCursor(if visible as i32 != 0 { 1 } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Down(button: MouseButton) -> bool {
    let button = 1 << button - 1;
    SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut()) & button as u32 > 0
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Pressed(button: MouseButton) -> bool {
    let button = 1 << button - 1;
    let current: u32 = SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut());
    current & button as u32 != 0 && lastState & button as u32 == 0
}

#[no_mangle]
pub unsafe extern "C" fn Mouse_Released(button: MouseButton) -> bool {
    let button = 1 << button - 1;
    let current: u32 = SDL_GetMouseState(std::ptr::null_mut(), std::ptr::null_mut());
    current & button as u32 == 0 && lastState & button as u32 != 0
}
