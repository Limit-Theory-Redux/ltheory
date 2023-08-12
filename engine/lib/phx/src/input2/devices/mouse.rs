use std::collections::HashMap;

use glam::Vec2;

use crate::{input2::*, internal::static_string, system::TimeStamp};

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
    ScrollX,
    ScrollY,
    ScrollLineX,
    ScrollLineY,
}

#[derive(Default)]
pub struct MouseState {
    control_state: ControlState,
    button_state: ButtonState<{ MouseControl::SIZE }>,
    axis_state: AxisState<{ MouseControl::SIZE }>,
}

impl MouseState {
    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    pub fn control_state_mut(&mut self) -> &mut ControlState {
        &mut self.control_state
    }

    pub fn reset(&mut self) {
        self.button_state.reset();
        self.axis_state.reset();
    }

    pub fn update_button(&mut self, control: MouseControl, pressed: bool) -> bool {
        self.button_state.update(control as _, pressed)
            && self
                .axis_state
                .update(control as _, if pressed { 1.0 } else { 0.0 })
            && self.control_state.update()
    }

    pub fn update_position_delta(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(MouseControl::DeltaX as _, x)
            && self.axis_state.update(MouseControl::DeltaY as _, y)
            && self.control_state.update()
    }

    pub fn update_scroll_pixel(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(MouseControl::ScrollX as _, x)
            && self.axis_state.update(MouseControl::ScrollY as _, y)
            && self.control_state.update()
    }

    pub fn update_scroll_line(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(MouseControl::ScrollLineX as _, x)
            && self.axis_state.update(MouseControl::ScrollLineY as _, y)
            && self.control_state.update()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl MouseState {
    pub fn value(&self, control: MouseControl) -> f32 {
        self.axis_state.value(control as _)
    }

    pub fn is_pressed(&self, control: MouseControl) -> bool {
        self.button_state.is_pressed(control as _)
    }

    pub fn is_down(&self, control: MouseControl) -> bool {
        self.button_state.is_down(control as _)
    }

    pub fn is_released(&self, control: MouseControl) -> bool {
        self.button_state.is_released(control as _)
    }

    pub fn delta(&self) -> Vec2 {
        let x = self.axis_state.value(MouseControl::DeltaX as _);
        let y = self.axis_state.value(MouseControl::DeltaY as _);

        Vec2::new(x, y)
    }

    pub fn scroll(&self) -> Vec2 {
        let x = self.axis_state.value(MouseControl::ScrollX as _);
        let y = self.axis_state.value(MouseControl::ScrollY as _);

        Vec2::new(x, y)
    }

    pub fn scroll_line(&self) -> Vec2 {
        let x = self.axis_state.value(MouseControl::ScrollLineX as _);
        let y = self.axis_state.value(MouseControl::ScrollLineY as _);

        Vec2::new(x, y)
    }
}
