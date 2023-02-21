use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    fn SDL_GetPerformanceCounter() -> u64;
    fn SDL_GetPerformanceFrequency() -> u64;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: u64,
}

static mut frequency: f64 = 0 as i32 as f64;
#[no_mangle]
pub unsafe extern "C" fn Timer_Create() -> *mut Timer {
    static mut init: bool = 0 as i32 != 0;
    if !init {
        init = 1 as i32 != 0;
        frequency = SDL_GetPerformanceFrequency() as f64;
    }
    let mut this: *mut Timer = MemAlloc(::core::mem::size_of::<Timer>() as usize) as *mut Timer;
    (*this).value = SDL_GetPerformanceCounter();
    return this;
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
    return elapsed;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_GetElapsed(mut this: *mut Timer) -> f64 {
    let mut now: u64 = SDL_GetPerformanceCounter();
    return now.wrapping_sub((*this).value) as f64 / frequency;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_Reset(mut this: *mut Timer) {
    (*this).value = SDL_GetPerformanceCounter();
}
