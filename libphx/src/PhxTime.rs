use crate::internal::Memory::*;
use glam::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Time {
    pub second: i32,
    pub minute: i32,
    pub hour: i32,
    pub dayOfWeek: i32,
    pub dayOfMonth: i32,
    pub dayOfYear: i32,
    pub month: i32,
    pub year: i32,
}

#[inline]
unsafe extern "C" fn Time_Convert(t: *const libc::tm) -> Time {
    let mut result: Time = Time {
        second: 0,
        minute: 0,
        hour: 0,
        dayOfWeek: 0,
        dayOfMonth: 0,
        dayOfYear: 0,
        month: 0,
        year: 0,
    };
    result.second = (*t).tm_sec;
    result.minute = (*t).tm_min;
    result.hour = (*t).tm_hour;
    result.dayOfWeek = (*t).tm_wday + 1_i32;
    result.dayOfMonth = (*t).tm_mday;
    result.dayOfYear = (*t).tm_yday + 1_i32;
    result.month = (*t).tm_mon + 1_i32;
    result.year = (*t).tm_year + 1900_i32;
    result
}

#[no_mangle]
pub unsafe extern "C" fn Time_GetLocal() -> Time {
    let mut t: libc::time_t = libc::time(std::ptr::null_mut());
    Time_Convert(libc::localtime(&mut t))
}

#[no_mangle]
pub unsafe extern "C" fn Time_GetUTC() -> Time {
    let mut t: libc::time_t = libc::time(std::ptr::null_mut());
    Time_Convert(libc::gmtime(&mut t))
}

#[no_mangle]
pub unsafe extern "C" fn Time_GetRaw() -> u32 {
    (libc::time(std::ptr::null_mut()) % 0xffffffff_u32 as libc::c_long) as u32
}
