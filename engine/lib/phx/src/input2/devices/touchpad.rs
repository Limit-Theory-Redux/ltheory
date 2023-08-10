use std::collections::HashMap;

use glam::Vec2;

use crate::{input2::*, internal::static_string, system::TimeStamp};

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadAxis {
    X,
    Y,
    MagnifyDelta,
    RotateDelta,
}

#[derive(Default)]
pub struct TouchpadState {
    control_state: ControlState,
    axis_state: AxisState<{ TouchpadAxis::SIZE }>,
}

impl TouchpadState {
    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    pub fn control_state_mut(&mut self) -> &mut ControlState {
        &mut self.control_state
    }

    pub fn reset(&mut self) {
        self.axis_state.reset();
    }

    pub fn update_position(&mut self, x: f32, y: f32) -> bool {
        self.axis_state.update(TouchpadAxis::X as usize, x)
            && self.axis_state.update(TouchpadAxis::Y as usize, y)
            && self.control_state.update()
    }

    pub fn update_magnify_delta(&mut self, value: f32) -> bool {
        self.axis_state
            .update(TouchpadAxis::MagnifyDelta as usize, value)
            && self.control_state.update()
    }

    pub fn update_rotate_delta(&mut self, value: f32) -> bool {
        self.axis_state
            .update(TouchpadAxis::RotateDelta as usize, value)
            && self.control_state.update()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TouchpadState {
    pub fn value(&self, axis: TouchpadAxis) -> f32 {
        self.axis_state.value(axis as usize)
    }

    pub fn position(&self) -> Vec2 {
        let x = self.axis_state.value(TouchpadAxis::X as usize);
        let y = self.axis_state.value(TouchpadAxis::Y as usize);

        Vec2::new(x, y)
    }

    pub fn magnify_delta(&self) -> f32 {
        self.axis_state.value(TouchpadAxis::MagnifyDelta as usize)
    }

    pub fn rotate_delta(&self) -> f32 {
        self.axis_state.value(TouchpadAxis::RotateDelta as usize)
    }
}
