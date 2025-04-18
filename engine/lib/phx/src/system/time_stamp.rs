use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Wrapper around Rust [`SystemTime`].
/// For documentation see: https://doc.rust-lang.org/std/time/struct.SystemTime.html
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeStamp {
    pub value: SystemTime,
}

impl std::fmt::Debug for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sec = self.to_seconds();
        let ms = self.to_subsec_millis();
        f.write_str(&format!("{sec}s {ms}ms"))
    }
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
    /// Get current timestamp.
    pub fn now() -> Self {
        Self {
            value: SystemTime::now(),
        }
    }

    /// Get timestamp `seconds` in a future from now.
    pub fn get_future(seconds: f64) -> Self {
        let d = Duration::from_secs_f64(seconds);

        Self {
            value: SystemTime::now()
                .checked_add(d)
                .expect("Cannot get future timestamp"),
        }
    }

    /// Get difference between 2 timestamps in double seconds.
    pub fn get_difference(&self, end_time: &TimeStamp) -> f64 {
        match end_time.value.duration_since(self.value) {
            Ok(difference) => difference.as_secs_f64(),
            Err(_e) => {
                let difference = self
                    .value
                    .duration_since(end_time.value)
                    .expect("Time overflow");
                -difference.as_secs_f64()
            }
        }
    }

    /// Number of seconds elapsed since this timestamp.
    pub fn get_elapsed(&self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        elapsed.as_secs_f64()
    }

    /// Number of milliseconds elapsed since this timestamp.
    pub fn get_elapsed_ms(&self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        elapsed.as_secs_f64() * 1000.0
    }

    /// Get timestamp `seconds` in a future relative to current one.
    pub fn get_relative(&self, seconds: f64) -> Self {
        let d = Duration::from_secs_f64(seconds);

        Self {
            value: self
                .value
                .checked_add(d)
                .expect("Cannot get relative timestamp"),
        }
    }

    /// Get duration since Unix epoch in double seconds.
    pub fn to_double(&self) -> f64 {
        let difference = self
            .value
            .duration_since(UNIX_EPOCH)
            .expect("Cannot convert timestamp to double");

        difference.as_secs_f64()
    }

    /// Get duration since Unix epoch in unsigned seconds.
    pub fn to_seconds(&self) -> u64 {
        let difference = self
            .value
            .duration_since(UNIX_EPOCH)
            .expect("Cannot convert timestamp to seconds");

        difference.as_secs()
    }

    /// Returns the fractional part in whole milliseconds.
    pub fn to_subsec_millis(&self) -> u32 {
        let difference = self
            .value
            .duration_since(UNIX_EPOCH)
            .expect("Cannot convert timestamp to seconds");

        difference.subsec_millis()
    }
}

impl From<SystemTime> for TimeStamp {
    fn from(value: SystemTime) -> Self {
        Self { value }
    }
}
