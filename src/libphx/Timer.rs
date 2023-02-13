use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn SDL_GetPerformanceCounter() -> Uint64;
    fn SDL_GetPerformanceFrequency() -> Uint64;
}
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
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
    let mut self_0: *mut Timer = MemAlloc(
        ::core::mem::size_of::<Timer>() as usize,
    ) as *mut Timer;
    (*self_0).value = SDL_GetPerformanceCounter();
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_Free(mut self_0: *mut Timer) {
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Timer_GetAndReset(mut self_0: *mut Timer) -> libc::c_double {
    let mut now: uint64 = SDL_GetPerformanceCounter();
    let mut elapsed: libc::c_double = now.wrapping_sub((*self_0).value) as libc::c_double
        / frequency;
    (*self_0).value = now;
    return elapsed;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_GetElapsed(mut self_0: *mut Timer) -> libc::c_double {
    let mut now: uint64 = SDL_GetPerformanceCounter();
    return now.wrapping_sub((*self_0).value) as libc::c_double / frequency;
}
#[no_mangle]
pub unsafe extern "C" fn Timer_Reset(mut self_0: *mut Timer) {
    (*self_0).value = SDL_GetPerformanceCounter();
}
