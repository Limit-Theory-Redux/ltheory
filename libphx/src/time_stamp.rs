use sdl2_sys::*;

pub type TimeStamp = u64;

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_Get() -> TimeStamp {
    SDL_GetPerformanceCounter()
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetDifference(start: TimeStamp, end: TimeStamp) -> f64 {
    let freq: f64 = SDL_GetPerformanceFrequency() as f64;
    end.wrapping_sub(start) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsed(then: TimeStamp) -> f64 {
    let freq: f64 = SDL_GetPerformanceFrequency() as f64;
    (SDL_GetPerformanceCounter()).wrapping_sub(then) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetElapsedMs(then: TimeStamp) -> f64 {
    let freq: f64 = SDL_GetPerformanceFrequency() as f64;
    1000_u64.wrapping_mul((SDL_GetPerformanceCounter()).wrapping_sub(then)) as f64 / freq
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetFuture(seconds: f64) -> TimeStamp {
    let freq: f64 = SDL_GetPerformanceFrequency() as f64;
    (SDL_GetPerformanceCounter()).wrapping_add((freq * seconds) as TimeStamp)
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_GetRelative(start: TimeStamp, seconds: f64) -> TimeStamp {
    let freq: f64 = SDL_GetPerformanceFrequency() as f64;
    start.wrapping_add((freq * seconds) as TimeStamp)
}

#[no_mangle]
pub unsafe extern "C" fn TimeStamp_ToDouble(this: TimeStamp) -> f64 {
    this as f64 / SDL_GetPerformanceFrequency() as f64
}
