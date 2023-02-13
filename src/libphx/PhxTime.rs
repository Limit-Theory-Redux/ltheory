use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn gmtime(_: *const time_t) -> *mut tm;
    fn localtime(_: *const time_t) -> *mut tm;
    fn time(_: *mut time_t) -> time_t;
}
pub type __darwin_time_t = libc::c_long;
pub type uint = libc::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Time {
    pub second: libc::c_int,
    pub minute: libc::c_int,
    pub hour: libc::c_int,
    pub dayOfWeek: libc::c_int,
    pub dayOfMonth: libc::c_int,
    pub dayOfYear: libc::c_int,
    pub month: libc::c_int,
    pub year: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tm {
    pub tm_sec: libc::c_int,
    pub tm_min: libc::c_int,
    pub tm_hour: libc::c_int,
    pub tm_mday: libc::c_int,
    pub tm_mon: libc::c_int,
    pub tm_year: libc::c_int,
    pub tm_wday: libc::c_int,
    pub tm_yday: libc::c_int,
    pub tm_isdst: libc::c_int,
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
    result.dayOfWeek = (*t).tm_wday + 1 as libc::c_int;
    result.dayOfMonth = (*t).tm_mday;
    result.dayOfYear = (*t).tm_yday + 1 as libc::c_int;
    result.month = (*t).tm_mon + 1 as libc::c_int;
    result.year = (*t).tm_year + 1900 as libc::c_int;
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
    return (time(0 as *mut time_t) % 0xffffffff as libc::c_uint as libc::c_long) as uint;
}
