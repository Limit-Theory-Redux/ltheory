use std::collections::HashMap;

use crate::{
    input2::{AxisState, ButtonState},
    internal::static_string,
};

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
    button_state: ButtonState<{ MouseControl::SIZE }>,
    axis_state: AxisState<{ MouseControl::SIZE }>,
}

impl MouseState {
    pub fn reset(&mut self) {
        self.button_state.reset();
        self.axis_state.reset();
    }

    pub fn update_button(&mut self, control: MouseControl, pressed: bool) -> bool {
        self.button_state.update(control as usize, pressed)
            && self
                .axis_state
                .update(control as usize, if pressed { 1.0 } else { 0.0 })
    }

    pub fn update_axis(&mut self, control: MouseControl, value: f32) -> bool {
        self.axis_state.update(control as usize, value)
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl MouseState {
    pub fn value(&self, control: MouseControl) -> f32 {
        self.axis_state.value(control as usize)
    }

    pub fn is_pressed(&self, control: MouseControl) -> bool {
        self.button_state.is_pressed(control as usize)
    }

    pub fn is_down(&self, control: MouseControl) -> bool {
        self.button_state.is_down(control as usize)
    }

    pub fn is_released(&self, control: MouseControl) -> bool {
        self.button_state.is_released(control as usize)
    }
}
