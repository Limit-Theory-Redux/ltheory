use gilrs::GamepadId;
use winit::event::DeviceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDeviceId {
    Winit(DeviceId),
    Gilrs(GamepadId),
}

impl From<DeviceId> for InputDeviceId {
    fn from(value: DeviceId) -> Self {
        Self::Winit(value)
    }
}

impl From<GamepadId> for InputDeviceId {
    fn from(value: GamepadId) -> Self {
        Self::Gilrs(value)
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl InputDeviceId {
    #[bind(role = "to_string")]
    pub fn to_string(&self) -> String {
        format!("{self:?}")
    }
}
