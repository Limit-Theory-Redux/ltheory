use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn gmtime(_: *const time_t) -> *mut tm;
    fn localtime(_: *const time_t) -> *mut tm;
    fn time(_: *mut time_t) -> time_t;
}
pub type __darwin_time_t = libc::c_long;
pub type uint = u32;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tm {
    pub tm_sec: i32,
    pub tm_min: i32,
    pub tm_hour: i32,
    pub tm_mday: i32,
    pub tm_mon: i32,
    pub tm_year: i32,
    pub tm_wday: i32,
    pub tm_yday: i32,
    pub tm_isdst: i32,
    pub tm_gmtoff: libc::c_long,
    pub tm_zone: *mut libc::c_char,
}
pub type time_t = __darwin_time_t;
#[inline]
unsafe extern "C" fn Time_Convert(mut t: *mut tm) -> Time {
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
    result.dayOfWeek = (*t).tm_wday + 1 as i32;
    result.dayOfMonth = (*t).tm_mday;
    result.dayOfYear = (*t).tm_yday + 1 as i32;
    result.month = (*t).tm_mon + 1 as i32;
    result.year = (*t).tm_year + 1900 as i32;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn Time_GetLocal() -> Time {
    let mut t: time_t = time(0 as *mut time_t);
    return Time_Convert(localtime(&mut t));
}
#[no_mangle]
pub unsafe extern "C" fn Time_GetUTC() -> Time {
    let mut t: time_t = time(0 as *mut time_t);
    return Time_Convert(gmtime(&mut t));
}
#[no_mangle]
pub unsafe extern "C" fn Time_GetRaw() -> uint {
    return (time(0 as *mut time_t) % 0xffffffff as u32 as libc::c_long) as uint;
}
