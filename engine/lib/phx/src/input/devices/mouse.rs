use std::collections::HashMap;

use glam::Vec2;

use crate::{input::*, system::TimeStamp};

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
    ScrollPixelX,
    ScrollPixelY,
}

impl MouseControl {
    pub fn is_button(&self) -> bool {
        match self {
            Self::Left | Self::Middle | Self::Right | Self::X1 | Self::X2 => true,
            _ => false,
        }
    }

    pub fn is_axis(&self) -> bool {
        match self {
            Self::DeltaX
            | Self::DeltaY
            | Self::ScrollX
            | Self::ScrollY
            | Self::ScrollPixelX
            | Self::ScrollPixelY => true,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct MouseState {
    control_state: ControlState,
    button_state: ButtonState<{ MouseControl::SIZE }>,
    axis_state: AxisState<{ MouseControl::SIZE }>,

    position: Vec2,
    in_window: bool,
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

    pub fn update_scroll_pixel(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(MouseControl::ScrollPixelX as _, x)
            && self.axis_state.update(MouseControl::ScrollPixelY as _, y)
            && self.control_state.update()
    }

    pub fn update_scroll_line(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(MouseControl::ScrollX as _, x)
            && self.axis_state.update(MouseControl::ScrollY as _, y)
            && self.control_state.update()
    }

    pub fn update_position(&mut self, x: f32, y: f32) -> bool {
        let prev_pos = self.position;

        self.position = Vec2::new(x, y);

        self.axis_state
            .update(MouseControl::DeltaX as _, x - prev_pos.x)
            && self
                .axis_state
                .update(MouseControl::DeltaY as _, y - prev_pos.y)
            && self.control_state.update()
    }

    pub fn update_in_window(&mut self, in_window: bool) -> bool {
        self.in_window = in_window;

        self.control_state.update()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl MouseState {
    pub fn value(&self, control: MouseControl) -> f32 {
        // TODO: should MouseControl be splitted to MouseButton and MouseAxis? This will require changes in Lua scripts
        if control.is_axis() {
            self.axis_state.value(control as _)
        } else if self.button_state.is_down(control as _) {
            1.0
        } else {
            0.0
        }
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

    pub fn scroll_pixel(&self) -> Vec2 {
        let x = self.axis_state.value(MouseControl::ScrollPixelX as _);
        let y = self.axis_state.value(MouseControl::ScrollPixelY as _);

        Vec2::new(x, y)
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn in_window(&self) -> bool {
        self.in_window
    }
}
