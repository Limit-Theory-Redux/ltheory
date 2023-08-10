use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceType {
    Cursor,
    Gamepad,
    Keyboard,
    Mouse,
    Touchpad,
}
