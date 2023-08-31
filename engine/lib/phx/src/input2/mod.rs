mod axis_state;
mod button;
mod button_state;
mod control_state;
mod device;
mod device_id;
mod device_type;
mod devices;
mod drag_and_drop;
mod system_event;
mod user_change;

pub use axis_state::*;
pub use button::*;
pub use button_state::*;
pub use control_state::*;
pub use device::*;
pub use device_id::*;
pub use device_type::*;
pub use devices::*;
pub use drag_and_drop::*;
pub use system_event::*;
pub use user_change::*;

use gilrs::GamepadId;
use winit::event::DeviceId;

use crate::system::TimeStamp;

#[derive(Default)]
pub struct Input {
    active_device: Option<InputDevice>,
    last_timestamp: TimeStamp,
    last_event_timestamp: TimeStamp,
    auto_hide_cursor: bool,
    user_changes: Vec<UserChange>,

    // TODO: add support for multiple devices per type (like with gamepad)
    cursor_state: CursorState,
    keyboard_state: KeyboardState,
    mouse_state: MouseState,
    touchpad_state: TouchpadState,
    gamepad_state: GamepadState,

    drag_and_drop_state: DragAndDropState,
}

impl Input {
    pub fn user_changes(&self) -> &[UserChange] {
        self.user_changes.as_ref()
    }

    pub fn reset(&mut self) {
        // self.active_device = None;

        self.user_changes.clear();

        self.keyboard_state.reset();
        self.mouse_state.reset();
        self.touchpad_state.reset();
        self.gamepad_state.reset();
        self.drag_and_drop_state.reset();

        self.last_timestamp = TimeStamp::now();
    }

    pub fn update_cursor(
        &mut self,
        device_id: DeviceId,
        mut f: impl FnMut(&mut CursorState) -> bool,
    ) -> bool {
        if f(&mut self.cursor_state) {
            self.set_active_device(InputDeviceType::Cursor, device_id);
            true
        } else {
            false
        }
    }

    pub fn update_keyboard(
        &mut self,
        device_id: DeviceId,
        mut f: impl FnMut(&mut KeyboardState) -> bool,
    ) -> bool {
        if f(&mut self.keyboard_state) {
            self.set_active_device(InputDeviceType::Keyboard, device_id);
            true
        } else {
            false
        }
    }

    pub fn update_mouse(
        &mut self,
        device_id: DeviceId,
        mut f: impl FnMut(&mut MouseState) -> bool,
    ) -> bool {
        if f(&mut self.mouse_state) {
            self.set_active_device(InputDeviceType::Mouse, device_id);
            true
        } else {
            false
        }
    }

    pub fn update_touchpad(
        &mut self,
        device_id: DeviceId,
        mut f: impl FnMut(&mut TouchpadState) -> bool,
    ) -> bool {
        if f(&mut self.touchpad_state) {
            self.set_active_device(InputDeviceType::Touchpad, device_id);
            true
        } else {
            false
        }
    }

    pub fn update_gamepad(
        &mut self,
        mut f: impl FnMut(&mut GamepadState) -> Option<GamepadId>,
    ) -> bool {
        if let Some(id) = f(&mut self.gamepad_state) {
            self.set_active_device(InputDeviceType::Gamepad, id);
            true
        } else {
            false
        }
    }

    pub fn update_drag_and_drop(
        &mut self,
        mut f: impl FnMut(&mut DragAndDropState) -> bool,
    ) -> bool {
        f(&mut self.drag_and_drop_state)
    }

    fn set_active_device(&mut self, ty: InputDeviceType, id: impl Into<InputDeviceId>) {
        self.active_device = Some(InputDevice { ty, id: id.into() });

        if self.auto_hide_cursor {
            self.user_changes
                .push(UserChange::CursorVisible(ty == InputDeviceType::Mouse));
        }

        self.last_timestamp = TimeStamp::now();
        self.last_event_timestamp = self.last_timestamp;
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Input {
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

    pub fn active_device(&self) -> Option<&InputDevice> {
        self.active_device.as_ref()
    }

    pub fn active_device_type(&self) -> Option<InputDeviceType> {
        self.active_device.as_ref().map(|dev| dev.ty)
    }

    pub fn active_device_id(&self) -> Option<&InputDeviceId> {
        self.active_device.as_ref().map(|dev| &dev.id)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.auto_hide_cursor = false;
        self.user_changes.push(UserChange::CursorVisible(visible));
    }

    pub fn set_cursor_visible_auto(&mut self) {
        self.auto_hide_cursor = true;

        if let Some(active_device) = &self.active_device {
            self.set_active_device(active_device.ty, active_device.id);
        }
    }

    pub fn set_cursor_position(&mut self, x: f32, y: f32) {
        self.user_changes.push(UserChange::CursorPosition(x, y));
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        if let Some(keyboard_button) = button.as_keyboard_button() {
            self.keyboard_state.is_pressed(keyboard_button)
        } else if let Some(mouse_control) = button.as_mouse_control() {
            self.mouse_state.is_pressed(mouse_control)
        } else if let Some(gamepad_button) = button.as_gamepad_button() {
            self.gamepad_state.is_pressed(gamepad_button)
        // TODO: should this be enabled?
        // } else if let Some(gamepad_axis) = button.as_gamepad_axis() {
        //     self.gamepad_state.value(gamepad_axis) != 0.0
        // } else if let Some(touchpad_axis) = button.as_touchpad_axis() {
        //     self.touchpad_state.value(touchpad_axis) != 0.0
        } else {
            false
        }
    }

    pub fn is_down(&self, button: Button) -> bool {
        if let Some(keyboard_button) = button.as_keyboard_button() {
            self.keyboard_state.is_down(keyboard_button)
        } else if let Some(mouse_control) = button.as_mouse_control() {
            self.mouse_state.is_down(mouse_control)
        } else if let Some(gamepad_button) = button.as_gamepad_button() {
            self.gamepad_state.is_down(gamepad_button)
        // TODO: should this be enabled?
        // } else if let Some(gamepad_axis) = button.as_gamepad_axis() {
        //     self.gamepad_state.value(gamepad_axis) != 0.0
        // } else if let Some(touchpad_axis) = button.as_touchpad_axis() {
        //     self.touchpad_state.value(touchpad_axis) != 0.0
        } else {
            false
        }
    }

    pub fn is_released(&self, button: Button) -> bool {
        if let Some(keyboard_button) = button.as_keyboard_button() {
            self.keyboard_state.is_released(keyboard_button)
        } else if let Some(mouse_control) = button.as_mouse_control() {
            self.mouse_state.is_released(mouse_control)
        } else if let Some(gamepad_button) = button.as_gamepad_button() {
            self.gamepad_state.is_released(gamepad_button)
        // TODO: should this be enabled?
        // } else if let Some(gamepad_axis) = button.as_gamepad_axis() {
        //     self.gamepad_state.value(gamepad_axis) == 0.0
        // } else if let Some(touchpad_axis) = button.as_touchpad_axis() {
        //     self.touchpad_state.value(touchpad_axis) == 0.0
        } else {
            false
        }
    }

    pub fn get_value(&self, button: Button) -> f32 {
        if let Some(keyboard_button) = button.as_keyboard_button() {
            self.keyboard_state.value(keyboard_button)
        } else if let Some(mouse_control) = button.as_mouse_control() {
            self.mouse_state.value(mouse_control)
        } else if let Some(gamepad_button) = button.as_gamepad_button() {
            self.gamepad_state
                .is_pressed(gamepad_button)
                .then_some(1.0)
                .unwrap_or_default()
        } else if let Some(gamepad_axis) = button.as_gamepad_axis() {
            self.gamepad_state.value(gamepad_axis)
        } else if let Some(touchpad_axis) = button.as_touchpad_axis() {
            self.touchpad_state.value(touchpad_axis)
        } else {
            0.0
        }
    }

    pub fn is_keyboard_alt_pressed(&self) -> bool {
        let keyboard = self.keyboard();

        keyboard.is_pressed(KeyboardButton::AltLeft)
            || keyboard.is_pressed(KeyboardButton::AltRight)
    }

    pub fn is_keyboard_ctrl_pressed(&self) -> bool {
        let keyboard = self.keyboard();

        keyboard.is_pressed(KeyboardButton::ControlLeft)
            || keyboard.is_pressed(KeyboardButton::ControlRight)
    }

    pub fn is_keyboard_shift_pressed(&self) -> bool {
        let keyboard = self.keyboard();

        keyboard.is_pressed(KeyboardButton::ShiftLeft)
            || keyboard.is_pressed(KeyboardButton::ShiftRight)
    }
}
