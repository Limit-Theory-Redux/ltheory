#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceType {
    Cursor,
    Gamepad,
    Keyboard,
    Mouse,
    Touchpad,
    SystemEvent,
}
