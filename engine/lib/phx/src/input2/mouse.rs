use std::collections::HashMap;

use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseControl {
    Left,
    Middle,
    Right,
    X1,
    X2,
    DeltaX,
    DeltaY,
    ScrollPixelX,
    ScrollPixelY,
    ScrollLineX,
    ScrollLineY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseControlFull {
    MouseControl(MouseControl),
    Other(u16), // TODO: can we expose this somehow to the Lua code?
}

#[derive(Default)]
pub struct MouseState {
    controls: [f32; MOUSE_CONTROL_COUNT],
    controls_other: HashMap<u16, f32>,
}

impl MouseState {
    pub fn reset(&mut self) {
        // Reset non-button controls only
        self.controls[MouseControl::X1 as usize] = 0.0;
        self.controls[MouseControl::X2 as usize] = 0.0;
        self.controls[MouseControl::DeltaX as usize] = 0.0;
        self.controls[MouseControl::DeltaY as usize] = 0.0;
        self.controls[MouseControl::ScrollPixelX as usize] = 0.0;
        self.controls[MouseControl::ScrollPixelY as usize] = 0.0;
        self.controls[MouseControl::ScrollLineX as usize] = 0.0;
        self.controls[MouseControl::ScrollLineY as usize] = 0.0;
    }

    pub fn update(&mut self, control: MouseControlFull, val: f32) {
        match control {
            MouseControlFull::MouseControl(control) => self.controls[control as usize] = val,
            MouseControlFull::Other(id) => {
                let _ = self.controls_other.insert(id, val);
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl MouseState {
    pub fn get_value(&self, control: MouseControl) -> f32 {
        if let Some(val) = self.controls.get(control as usize) {
            *val
        } else if let Some(val) = self.controls_other.get(&(control as u16)) {
            *val
        } else {
            0.0 // TODO: return an error?
        }
    }
}
