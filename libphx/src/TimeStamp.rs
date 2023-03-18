use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

pub type TimeStamp = u64;

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_Get() -> TimeStamp {
    SDL_GetPerformanceCounter()
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetDifference(mut start: TimeStamp, mut end: TimeStamp) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    end.wrapping_sub(start) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsed(mut then: TimeStamp) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    (SDL_GetPerformanceCounter()).wrapping_sub(then) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsedMs(mut then: TimeStamp) -> f64 {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    1000_u64.wrapping_mul((SDL_GetPerformanceCounter()).wrapping_sub(then)) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetFuture(mut seconds: f64) -> TimeStamp {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    (SDL_GetPerformanceCounter()).wrapping_add((freq * seconds) as TimeStamp)
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetRelative(
    mut start: TimeStamp,
    mut seconds: f64,
) -> TimeStamp {
    let mut freq: f64 = SDL_GetPerformanceFrequency() as f64;
    start.wrapping_add((freq * seconds) as TimeStamp)
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_ToDouble(mut this: TimeStamp) -> f64 {
    this as f64 / SDL_GetPerformanceFrequency() as f64
}
