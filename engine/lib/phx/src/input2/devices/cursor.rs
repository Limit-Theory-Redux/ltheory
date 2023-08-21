use std::collections::HashMap;

use glam::Vec2;

use crate::{input2::ControlState, system::TimeStamp};

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorControl {
    X,
    Y,
    InWindow,
}

#[derive(Default)]
pub struct CursorState {
    control_state: ControlState,

    x: f32,
    y: f32,
    in_window: bool,
}

impl CursorState {
    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    pub fn control_state_mut(&mut self) -> &mut ControlState {
        &mut self.control_state
    }

    pub fn update_position(&mut self, x: f32, y: f32) -> bool {
        self.x = x;
        self.y = y;

        self.control_state.update()
    }

    pub fn update_in_window(&mut self, in_window: bool) -> bool {
        self.in_window = in_window;

        self.control_state.update()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl CursorState {
    pub fn value(&self, control: CursorControl) -> f32 {
        match control {
            CursorControl::X => self.x,
            CursorControl::Y => self.y,
            CursorControl::InWindow => {
                if self.in_window {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn in_window(&self) -> bool {
        self.in_window
    }
}
