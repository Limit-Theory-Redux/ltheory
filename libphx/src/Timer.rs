use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn SDL_GetPerformanceCounter() -> Uint64;
    fn SDL_GetPerformanceFrequency() -> Uint64;
}
pub type uint64_t = libc::c_ulonglong;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: uint64,
}
pub type Uint64 = uint64_t;

static mut frequency: libc::c_double = 0 as libc::c_int as libc::c_double;
#[no_mangle]
pub unsafe extern "C" fn Timer_Create() -> *mut Timer {
    static mut init: bool = 0 as libc::c_int != 0;
    if !init {
        init = 1 as libc::c_int != 0;
        frequency = SDL_GetPerformanceFrequency() as libc::c_double;
    }
    let mut this: *mut Timer = MemAlloc(
        ::core::mem::size_of::<Timer>() as usize,
    ) as *mut Timer;
    (*this).value = SDL_GetPerformanceCounter();
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_Free(mut this: *mut Timer) {
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Timer_GetAndReset(mut this: *mut Timer) -> libc::c_double {
    let mut now: uint64 = SDL_GetPerformanceCounter();
    let mut elapsed: libc::c_double = now.wrapping_sub((*this).value) as libc::c_double
        / frequency;
    (*this).value = now;
    return elapsed;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_GetElapsed(mut this: *mut Timer) -> libc::c_double {
    let mut now: uint64 = SDL_GetPerformanceCounter();
    return now.wrapping_sub((*this).value) as libc::c_double / frequency;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_Reset(mut this: *mut Timer) {
    (*this).value = SDL_GetPerformanceCounter();
}
