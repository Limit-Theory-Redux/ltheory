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

#[derive(Default)]
pub struct MouseState {
    transitions: [u32; MOUSE_CONTROL_COUNT],
    buttons: [bool; MOUSE_CONTROL_COUNT],
    axes: [f32; MOUSE_CONTROL_COUNT],
}

impl MouseState {
    pub fn reset(&mut self) {
        self.transitions.fill(0);

        // Reset non-button controls only
        self.axes[MouseControl::DeltaX as usize] = 0.0;
        self.axes[MouseControl::DeltaY as usize] = 0.0;
        self.axes[MouseControl::ScrollPixelX as usize] = 0.0;
        self.axes[MouseControl::ScrollPixelY as usize] = 0.0;
        self.axes[MouseControl::ScrollLineX as usize] = 0.0;
        self.axes[MouseControl::ScrollLineY as usize] = 0.0;
    }

    pub fn pressed(&mut self, control: MouseControl) {
        self.buttons[control as usize] = true;
        self.transitions[control as usize] += 1;
    }

    pub fn released(&mut self, control: MouseControl) {
        self.buttons[control as usize] = false;
        self.transitions[control as usize] += 1;
    }

    pub fn update(&mut self, control: MouseControl, val: f32) {
        self.axes[control as usize] = val;
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl MouseState {
    pub fn get_value(&self, control: MouseControl) -> f32 {
        if let Some(val) = self.axes.get(control as usize) {
            *val
        } else {
            0.0 // TODO: return an error?
        }
    }

    pub fn is_pressed(&self, control: MouseControl) -> bool {
        let index = control as usize;

        self.buttons[index] || self.transitions[index] > 0
    }

    pub fn is_down(&self, control: MouseControl) -> bool {
        let index = control as usize;

        if self.buttons[index] {
            self.transitions[index] > 0
        } else {
            self.transitions[index] > 1
        }
    }

    pub fn is_released(&self, control: MouseControl) -> bool {
        let index = control as usize;

        if self.buttons[index] {
            self.transitions[index] > 1
        } else {
            self.transitions[index] > 0
        }
    }
}
