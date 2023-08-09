use std::collections::HashMap;

use glam::Vec2;

use crate::{input2::AxisState, internal::static_string};

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
    axis_state: AxisState<{ TouchpadAxis::SIZE }>,
}

impl TouchpadState {
    pub fn reset(&mut self) {
        self.axis_state.reset();
    }

    pub fn update(&mut self, axis: TouchpadAxis, value: f32) -> bool {
        self.axis_state.update(axis as usize, value)
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
