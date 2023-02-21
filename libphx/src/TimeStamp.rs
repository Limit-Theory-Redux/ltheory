use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn SDL_GetPerformanceFrequency() -> Uint64;
    fn SDL_GetPerformanceCounter() -> Uint64;
}
pub type uint64_t = libc::c_ulonglong;
pub type Uint64 = uint64_t;
pub type uint64 = uint64_t;
pub type TimeStamp = uint64;
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_Get() -> TimeStamp {
    return SDL_GetPerformanceCounter();
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetDifference(
    mut start: TimeStamp,
    mut end: TimeStamp,
) -> libc::c_double {
    let mut freq: libc::c_double = SDL_GetPerformanceFrequency() as libc::c_double;
    return end.wrapping_sub(start) as libc::c_double / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsed(mut then: TimeStamp) -> libc::c_double {
    let mut freq: libc::c_double = SDL_GetPerformanceFrequency() as libc::c_double;
    return (SDL_GetPerformanceCounter()).wrapping_sub(then) as libc::c_double / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsedMs(mut then: TimeStamp) -> libc::c_double {
    let mut freq: libc::c_double = SDL_GetPerformanceFrequency() as libc::c_double;
    return (1000 as libc::c_ulonglong)
        .wrapping_mul((SDL_GetPerformanceCounter()).wrapping_sub(then)) as libc::c_double
        / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetFuture(mut seconds: libc::c_double) -> TimeStamp {
    let mut freq: libc::c_double = SDL_GetPerformanceFrequency() as libc::c_double;
    return (SDL_GetPerformanceCounter()).wrapping_add((freq * seconds) as TimeStamp);
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetRelative(
    mut start: TimeStamp,
    mut seconds: libc::c_double,
) -> TimeStamp {
    let mut freq: libc::c_double = SDL_GetPerformanceFrequency() as libc::c_double;
    return start.wrapping_add((freq * seconds) as TimeStamp);
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_ToDouble(mut this: TimeStamp) -> libc::c_double {
    return this as libc::c_double / SDL_GetPerformanceFrequency() as libc::c_double;
}
