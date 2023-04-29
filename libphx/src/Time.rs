use chrono::{DateTime, Utc, Local, TimeZone, Timelike, Datelike};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn TimeFromChrono<T: TimeZone>(dt: DateTime<T>) -> Time {
    let time = dt.time();
    let date = dt.date_naive();
    Time {
        second: time.second() as i32,
        minute: time.minute() as i32,
        hour: time.hour() as i32,
        dayOfWeek: date.weekday().num_days_from_sunday() as i32,
        dayOfMonth: date.day() as i32,
        dayOfYear: date.ordinal() as i32,
        month: date.month() as i32,
        year: date.year() as i32,
    }
}

#[no_mangle]
pub extern "C" fn Time_GetLocal() -> Time {
    TimeFromChrono(Local::now())
}

#[no_mangle]
pub extern "C" fn Time_GetUTC() -> Time {
    TimeFromChrono(Utc::now())
}

// Seconds since epoch.
#[no_mangle]
pub extern "C" fn Time_GetRaw() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}
