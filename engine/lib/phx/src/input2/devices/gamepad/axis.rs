use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    /// The horizontal value of the left stick.
    LeftStickX,
    /// The vertical value of the left stick.
    LeftStickY,
    /// The value of the left `Z` button.
    LeftZ,

    /// The horizontal value of the right stick.
    RightStickX,
    /// The vertical value of the right stick.
    RightStickY,
    /// The value of the right `Z` button.
    RightZ,
}
