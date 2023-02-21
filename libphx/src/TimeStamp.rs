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
) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    return end.wrapping_sub(start) as f64 / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsed(mut then: TimeStamp) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    return (SDL_GetPerformanceCounter()).wrapping_sub(then) as f64 / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsedMs(mut then: TimeStamp) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    return (1000 as libc::c_ulonglong)
        .wrapping_mul((SDL_GetPerformanceCounter()).wrapping_sub(then)) as f64
        / freq;
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetFuture(mut seconds: f64) -> TimeStamp {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    return (SDL_GetPerformanceCounter()).wrapping_add((freq * seconds) as TimeStamp);
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetRelative(
    mut start: TimeStamp,
    mut seconds: f64,
) -> TimeStamp {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    return start.wrapping_add((freq * seconds) as TimeStamp);
}
#[no_mangle]
pub unsafe extern "C" fn TimeStamp_ToDouble(mut this: TimeStamp) -> f64 {
    return this as f64 / SDL_GetPerformanceFrequency() as f64;
}
