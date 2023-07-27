use std::collections::HashMap;

use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadControl {
    X,
    Y,
    MagnifyDelta,
    RotateDelta,
}

#[derive(Default)]
pub struct TouchpadState {
    controls: [f32; TOUCHPAD_CONTROL_COUNT],
}

impl TouchpadState {
    pub fn reset(&mut self) {
        self.controls[TouchpadControl::MagnifyDelta as usize] = 0.0;
        self.controls[TouchpadControl::RotateDelta as usize] = 0.0;
    }

    pub fn update(&mut self, control: TouchpadControl, val: f32) {
        self.controls[control as usize] = val;
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TouchpadState {
    pub fn get_value(&self, control: TouchpadControl) -> f32 {
        if let Some(val) = self.controls.get(control as usize) {
            *val
        } else {
            0.0 // TODO: return an error?
        }
    }
}
