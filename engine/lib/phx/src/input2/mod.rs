use self::mouse::MouseState;

mod cursor;
mod gamepad;
mod keyboard;
mod mouse;
mod touchpad;

pub use cursor::*;
pub use gamepad::*;
pub use keyboard::*;
pub use mouse::*;
pub use touchpad::*;

#[derive(Default)]
pub struct Input2 {
    pub cursor_state: CursorState,
    pub keyboard_state: KeyboardState,
    pub mouse_state: MouseState,
    pub touchpad_state: TouchpadState,
}

impl Input2 {
    pub fn reset(&mut self) {
        self.mouse_state.reset();
        self.touchpad_state.reset();
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Input2 {
    pub fn cursor(&self) -> &CursorState {
        &self.cursor_state
    }

    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard_state
    }

    pub fn mouse(&self) -> &MouseState {
        &self.mouse_state
    }

    pub fn touchpad(&self) -> &TouchpadState {
        &self.touchpad_state
    }
}
