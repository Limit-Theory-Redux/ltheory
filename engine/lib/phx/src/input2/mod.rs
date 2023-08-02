use self::mouse::MouseState;

mod cursor;
mod drag_and_drop;
mod gamepad;
mod keyboard;
mod mouse;
mod touchpad;

pub use cursor::*;
pub use drag_and_drop::*;
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
    pub gamepad_state: GamepadState,
    pub drag_and_drop_state: DragAndDropState,
}

impl Input2 {
    pub fn reset(&mut self) {
        self.mouse_state.reset();
        self.touchpad_state.reset();
        self.gamepad_state.reset();
        self.drag_and_drop_state.reset();
    }

    pub fn gamepad_mut(&mut self) -> &mut GamepadState {
        &mut self.gamepad_state
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

    pub fn gamepad(&self) -> &GamepadState {
        &self.gamepad_state
    }

    pub fn drag_and_drop(&self) -> &DragAndDropState {
        &self.drag_and_drop_state
    }
}
