use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

pub type TimeStamp = u64;

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_Get() -> TimeStamp {
    return SDL_GetPerformanceCounter();
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetDifference(mut start: TimeStamp, mut end: TimeStamp) -> f64 {
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
    return 1000_u64.wrapping_mul((SDL_GetPerformanceCounter()).wrapping_sub(then)) as f64
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
