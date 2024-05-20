use std::time::Instant;

pub struct InstantTime {
    time: Instant,
}

#[luajit_ffi_gen::luajit_ffi]
impl InstantTime {
    /// Return time in double milliseconds passed since earlier time.
    pub fn duration_since(&self, earlier: &Self) -> f64 {
        self.time.duration_since(earlier.time).as_secs_f64() * 1000.0
    }
}

impl From<Instant> for InstantTime {
    fn from(time: Instant) -> Self {
        Self { time }
    }
}
