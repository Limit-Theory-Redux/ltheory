use std::time::SystemTime;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: SystemTime,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            value: SystemTime::now(),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Timer {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_and_reset(&mut self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        self.value += elapsed;

        elapsed.as_secs_f64()
    }

    pub fn get_elapsed(&self) -> f64 {
        let elapsed = self.value.elapsed().expect("Cannot get elapsed time");

        elapsed.as_secs_f64()
    }

    pub fn reset(&mut self) {
        self.value = SystemTime::now();
    }
}
