use std::collections::HashMap;

use gilrs::{ev::filter::axis_dpad_to_button, EventType, Filter, Gilrs, GilrsBuilder};

use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GamepadId {
    pub id: usize,
}

impl GamepadId {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Default)]
pub struct GamepadDeviceState {
    buttons: [f32; GAMEPAD_BUTTON_COUNT],
    axes: [f32; GAMEPAD_AXIS_COUNT],
}

pub struct GamepadState {
    gilrs: Gilrs,
    device_state: HashMap<GamepadId, GamepadDeviceState>,
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
            .for_each(|(_, state)| state.axes = Default::default());
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
                    self.device_state.insert(gamepad_id, Default::default());
                }
                EventType::Disconnected => {
                    self.device_state.remove(&gamepad_id);
                }
                EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                    if let Some(button) = convert_button(gilrs_button) {
                        let mut states = self.device_state.entry(gamepad_id).or_default();

                        states.buttons[button as usize] = raw_value;

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
                        let mut states = self.device_state.entry(gamepad_id).or_default();

                        states.axes[axis as usize] = raw_value;

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
    pub fn get_gamepads_count(&self) -> usize {
        self.device_state.len()
    }

    pub fn get_gamepad_id(&self, index: usize) -> Option<GamepadId> {
        let mut device_ids: Vec<_> = self.device_state.keys().collect();

        device_ids.sort(); // TODO: do we really need this?

        device_ids.get(index).map(|id| **id)
    }

    pub fn get_button_value(&self, gamepad_id: GamepadId, button: GamepadButton) -> f32 {
        if let Some(state) = self.device_state.get(&gamepad_id) {
            state
                .buttons
                .get(button as usize)
                .map(|val| *val)
                .unwrap_or_default()
        } else {
            0.0 // TODO: return an error?
        }
    }

    pub fn get_axis_value(&self, gamepad_id: GamepadId, axis: GamepadAxis) -> f32 {
        if let Some(state) = self.device_state.get(&gamepad_id) {
            state
                .axes
                .get(axis as usize)
                .map(|val| *val)
                .unwrap_or_default()
        } else {
            0.0 // TODO: return an error?
        }
    }
}
