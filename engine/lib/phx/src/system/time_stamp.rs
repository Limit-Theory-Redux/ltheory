use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct TimeStamp {
    pub value: SystemTime,
}

impl Default for TimeStamp {
    fn default() -> Self {
        Self::zero()
    }
}

impl TimeStamp {
    pub const fn zero() -> Self {
        Self { value: UNIX_EPOCH }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TimeStamp {
    pub fn now() -> Self {
        Self {
            value: SystemTime::now(),
        }
    }

    pub fn get_future(seconds: f64) -> Self {
        let d = Duration::from_secs_f64(seconds);

        Self {
            value: SystemTime::now()
                .checked_add(d)
                .expect("Cannot get future timestamp"),
        }
    }

    pub fn get_difference(&self, end_time: &TimeStamp) -> f64 {
        let difference = end_time
            .value
            .duration_since(self.value)
            .expect("Cannot get timestamp difference");

        difference.as_secs_f64()
    }

    /// Number of seconds elapsed since this timestamp.
    pub fn get_elapsed(&self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        elapsed.as_secs_f64()
    }

    pub fn get_elapsed_ms(&self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        elapsed.as_secs_f64() * 1000.0
    }

    pub fn get_relative(&self, seconds: f64) -> Self {
        let d = Duration::from_secs_f64(seconds);

        Self {
            value: self
                .value
                .checked_add(d)
                .expect("Cannot get relative timestamp"),
        }
    }

    pub fn to_double(&self) -> f64 {
        let difference = self
            .value
            .duration_since(UNIX_EPOCH)
            .expect("Cannot convert timestamp to double");

        difference.as_secs_f64()
    }

    pub fn to_seconds(&self) -> u64 {
        let difference = self
            .value
            .duration_since(UNIX_EPOCH)
            .expect("Cannot convert timestamp to seconds");

        difference.as_secs()
    }
}

impl From<SystemTime> for TimeStamp {
    fn from(value: SystemTime) -> Self {
        Self { value }
    }
}
