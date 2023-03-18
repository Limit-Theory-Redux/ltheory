use crate::internal::Memory::*;
use crate::Math::Vec3;
use libc;
use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: u64,
}

static mut frequency: f64 = 0_i32 as f64;

#[no_mangle]
pub unsafe extern "C" fn Timer_Create() -> *mut Timer {
    static mut init: bool = false;
    if !init {
        init = true;
        frequency = SDL_GetPerformanceFrequency() as f64;
    }
    let mut this: *mut Timer = MemAlloc(::core::mem::size_of::<Timer>()) as *mut Timer;
    (*this).value = SDL_GetPerformanceCounter();
    this
}

#[no_mangle]
pub unsafe extern "C" fn Timer_Free(mut this: *mut Timer) {
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Timer_GetAndReset(mut this: *mut Timer) -> f64 {
    let mut now: u64 = SDL_GetPerformanceCounter();
    let mut elapsed: f64 = now.wrapping_sub((*this).value) as f64 / frequency;
    (*this).value = now;
    elapsed
}

#[no_mangle]
pub unsafe extern "C" fn Timer_GetElapsed(mut this: *mut Timer) -> f64 {
    let mut now: u64 = SDL_GetPerformanceCounter();
    now.wrapping_sub((*this).value) as f64 / frequency
}

#[no_mangle]
pub unsafe extern "C" fn Timer_Reset(mut this: *mut Timer) {
    (*this).value = SDL_GetPerformanceCounter();
}
