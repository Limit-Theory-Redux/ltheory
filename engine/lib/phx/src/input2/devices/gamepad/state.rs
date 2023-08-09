use std::collections::HashMap;

use gilrs::{ev::filter::axis_dpad_to_button, EventType, Filter, Gilrs, GilrsBuilder};
use indexmap::IndexMap;

use super::*;
use crate::{
    input2::{AxisState, ButtonState},
    internal::static_string,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GamepadId {
    pub id: usize,
}

impl GamepadId {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

pub struct GamepadDeviceState {
    name: String,
    button_state: ButtonState<{ GamepadButton::SIZE }>,
    axis_state: AxisState<{ GamepadAxis::SIZE }>,
}

impl GamepadDeviceState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            button_state: Default::default(),
            axis_state: Default::default(),
        }
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
            // .add_mappings(include_str!(concat!(
            //     env!("CARGO_MANIFEST_DIR"),
            //     "/../../../res/gamecontrollerdb_2016.txt" // TODO: is there more convenient way to point to this file (workspace path)?
            // )))
            .build()
            .expect("Cannot create Gilrs"); // TODO: return Result

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

    pub fn update(&mut self) {
        while let Some(gilrs_event) = self
            .gilrs
            .next_event()
            .filter_ev(&axis_dpad_to_button, &mut self.gilrs)
        {
            self.gilrs.update(&gilrs_event);

            let gamepad_id = GamepadId::new(gilrs_event.id.into());
            match gilrs_event.event {
                EventType::Connected => {
                    let pad = self.gilrs.gamepad(gilrs_event.id);
                    let device_state = GamepadDeviceState::new(pad.name());

                    self.device_state.insert(gamepad_id, device_state);
                }
                EventType::Disconnected => {
                    self.device_state.remove(&gamepad_id);
                }
                EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                    if let Some(button) = convert_button(gilrs_button) {
                        if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                            state.button_state.update(button as usize, raw_value != 0.0);
                            state.axis_state.update(button as usize, raw_value);
                        }

                        // TODO: threshold check?
                        // let button = GamepadButton::new(gamepad_id, button_type);
                        // let old_value = gamepad_buttons.get(button);
                        // let button_settings = gamepad_settings.get_button_axis_settings(button);

                        // // Only send events that pass the user-defined change threshold
                        // if let Some(filtered_value) = button_settings.filter(raw_value, old_value) {
                        //     events.send(
                        //         GamepadButtonChangedEvent::new(
                        //             gamepad_id,
                        //             button_type,
                        //             filtered_value,
                        //         )
                        //         .into(),
                        //     );
                        // }
                    }
                }
                EventType::AxisChanged(gilrs_axis, raw_value, _) => {
                    if let Some(axis) = convert_axis(gilrs_axis) {
                        if let Some(state) = self.device_state.get_mut(&gamepad_id) {
                            state.axis_state.update(axis as usize, raw_value);
                        }

                        // TODO: threshold check?
                        // let axis = GamepadAxis::new(gamepad_id, axis_type);
                        // let old_value = gamepad_axis.get(axis);
                        // let axis_settings = gamepad_settings.get_axis_settings(axis);

                        // // Only send events that pass the user-defined change threshold
                        // if let Some(filtered_value) = axis_settings.filter(raw_value, old_value) {
                        //     events.send(
                        //         GamepadAxisChangedEvent::new(gamepad_id, axis_type, filtered_value)
                        //             .into(),
                        //     );
                        // }
                    }
                }
                _ => (),
            };
        }

        self.gilrs.inc();
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl GamepadState {
    pub fn gamepads_count(&self) -> usize {
        self.device_state.len()
    }

    pub fn gamepad_id(&self, index: usize) -> Option<GamepadId> {
        let mut device_ids: Vec<_> = self.device_state.keys().collect();

        device_ids.sort(); // TODO: do we really need this?

        device_ids.get(index).map(|id| **id)
    }

    pub fn gamepad_name(&self, gamepad_id: GamepadId) -> Option<String> {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.name.clone())
    }

    pub fn value(&self, gamepad_id: GamepadId, axis: GamepadAxis) -> f32 {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.axis_state.value(axis as usize))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_pressed(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_pressed(button as usize))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_down(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_down(button as usize))
            .unwrap_or_default() // TODO: return an error?
    }

    pub fn is_released(&self, gamepad_id: GamepadId, button: GamepadButton) -> bool {
        self.device_state
            .get(&gamepad_id)
            .map(|state| state.button_state.is_released(button as usize))
            .unwrap_or_default() // TODO: return an error?
    }
}
