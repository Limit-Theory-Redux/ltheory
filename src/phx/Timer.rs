use crate::phx::internal::Memory::*;


use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: u64,
}

static mut frequency: f64 = 0.0;

#[no_mangle]
pub unsafe extern "C" fn Timer_Create() -> *mut Timer {
    static mut init: bool = false;
    if !init {
        init = true;
        frequency = SDL_GetPerformanceFrequency() as f64;
    }
    let this = MemNew!(Timer);
    (*this).value = SDL_GetPerformanceCounter();
    this
}

#[no_mangle]
pub unsafe extern "C" fn Timer_Free(this: *mut Timer) {
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Timer_GetAndReset(this: &mut Timer) -> f64 {
    let now: u64 = SDL_GetPerformanceCounter();
    let elapsed: f64 = now.wrapping_sub(this.value) as f64 / frequency;
    this.value = now;
    elapsed
}

#[no_mangle]
pub unsafe extern "C" fn Timer_GetElapsed(this: &mut Timer) -> f64 {
    let now: u64 = SDL_GetPerformanceCounter();
    now.wrapping_sub(this.value) as f64 / frequency
}

#[no_mangle]
pub unsafe extern "C" fn Timer_Reset(this: &mut Timer) {
    this.value = SDL_GetPerformanceCounter();
}
