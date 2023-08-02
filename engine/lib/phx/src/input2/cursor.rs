use std::collections::HashMap;

use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorControl {
    X,
    Y,
    InWindow,
}

#[derive(Default)]
pub struct CursorState {
    controls: [f32; CURSOR_CONTROL_COUNT],
}

impl CursorState {
    pub fn update(&mut self, control: CursorControl, val: f32) {
        self.controls[control as usize] = val;
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl CursorState {
    pub fn get_value(&self, control: CursorControl) -> f32 {
        self.controls
            .get(control as usize)
            .map(|val| *val)
            .unwrap_or_default() // TODO: return an error?
    }
}
