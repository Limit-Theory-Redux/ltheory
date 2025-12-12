use gilrs::ev::filter::axis_dpad_to_button;
use gilrs::{EventType, Filter, GamepadId, Gilrs, GilrsBuilder};
use indexmap::IndexMap;

use super::*;
use crate::input::*;

pub struct GamepadDeviceState {
    name: String,

    control_state: ControlState,
    button_state: ButtonState<{ GamepadButton::SIZE }>,
    axis_state: AxisState<{ GamepadAxis::SIZE }>,
}

impl GamepadDeviceState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            control_state: Default::default(),
            button_state: Default::default(),
            axis_state: Default::default(),
        }
    }

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
}

pub struct GamepadState {
    gilrs: Gilrs,
    device_state: IndexMap<GamepadId, GamepadDeviceState>,
}

impl Default for GamepadState {
    fn default() -> Self {
        let gilrs = GilrsBuilder::new()
            .add_included_mappings(true) // Load the latest mappings from GitHub
            .build()
            .expect("Cannot create Gilrs");

        Self {
            gilrs,
            device_state: Default::default(),
        }
    }
}

impl GamepadState {
    pub fn reset(&mut self) {
        self.device_state
            .iter_mut()
            .for_each(|(_, state)| state.reset());
    }

    pub fn update(&mut self) -> Option<GamepadId> {
        let mut res = None;

        while let Some(gilrs_event) = self
            .gilrs
            .next_event()
            .filter_ev(&axis_dpad_to_button, &mut self.gilrs)
        {
            self.gilrs.update(&gilrs_event);

            let gamepad_id = gilrs_event.id;

            match gilrs_event.event {
                EventType::Connected => {
                    if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                        state.reset();
                        state.control_state_mut().connect();
                    } else {
                        let pad = self.gilrs.gamepad(gilrs_event.id);
                        let device_state = GamepadDeviceState::new(pad.name());

                        self.device_state.insert(gamepad_id, device_state);
                    }
                }
                EventType::Disconnected => {
                    if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                        state.control_state_mut().disconnect();
                    } else {
                        // TODO: warning?
                    }
                }
                EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                    if let Some(button) = convert_button(gilrs_button) {
                        if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                            state.button_state.update(button as _, raw_value != 0.0);
                            state.axis_state.update(button as _, raw_value);

                            if state.control_state.update() {
                                res = Some(gamepad_id);
                            }
                        }
                    }
                }
                EventType::AxisChanged(gilrs_axis, raw_value, _) => {
                    if let Some(axis) = convert_axis(gilrs_axis) {
                        if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                            state.axis_state.update(axis as _, raw_value);

                            if state.control_state.update() {
                                res = Some(gamepad_id);
                            }
                        }
                    }
                }
                _ => (),
            };
        }

        self.gilrs.inc();

        res
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl GamepadState {
    pub fn gamepads_count(&self) -> usize {
        self.device_state.len()
    }

    pub fn gamepad_id(&self, index: usize) -> Option<GamepadId> {
        let device_ids: Vec<_> = self.device_state.keys().collect();

        device_ids.get(index).map(|id| **id)
    }

    pub fn gamepad_name(&self, gamepad_id: GamepadId) -> Option<String> {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.name.clone())
    }

    pub fn value(&self, axis: GamepadAxis) -> f32 {
        self.device_state
            .iter()
            .find_map(|(_, state)| {
                state
                    .control_state
                    .is_connected()
                    .then(|| state.axis_state.value(axis as _))
            })
            .unwrap_or_default()
    }

    pub fn delta(&self, axis: GamepadAxis) -> f32 {
        self.device_state
            .iter()
            .find_map(|(_, state)| {
                state
                    .control_state
                    .is_connected()
                    .then(|| state.axis_state.delta(axis as _))
            })
            .unwrap_or_default()
    }

    pub fn is_pressed(&self, button: GamepadButton) -> bool {
        self.device_state
            .iter()
            .find_map(|(_, state)| {
                state
                    .control_state
                    .is_connected()
                    .then(|| state.button_state.is_pressed(button as _))
            })
            .unwrap_or_default()
    }

    pub fn is_down(&self, button: GamepadButton) -> bool {
        self.device_state
            .iter()
            .find_map(|(_, state)| {
                state
                    .control_state
                    .is_connected()
                    .then(|| state.button_state.is_down(button as _))
            })
            .unwrap_or_default()
    }

    pub fn is_released(&self, button: GamepadButton) -> bool {
        self.device_state
            .iter()
            .find_map(|(_, state)| {
                state
                    .control_state
                    .is_connected()
                    .then(|| state.button_state.is_released(button as _))
            })
            .unwrap_or_default()
    }

    pub fn value_by_id(&self, gamepad_id: GamepadId, axis: GamepadAxis) -> f32 {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.axis_state.value(axis as _))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn delta_by_id(&self, gamepad_id: GamepadId, axis: GamepadAxis) -> f32 {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.axis_state.delta(axis as _))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_pressed_by_id(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_pressed(button as _))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_down_by_id(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_down(button as _))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_released_by_id(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_released(button as _))
            .unwrap_or_default() // TODO: return an error?
    }
}
