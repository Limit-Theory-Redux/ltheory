use std::time::SystemTime;

use crate::internal::*;
use crate::*;

use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub value: SystemTime,
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Timer {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self {
            value: SystemTime::now(),
        }
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
