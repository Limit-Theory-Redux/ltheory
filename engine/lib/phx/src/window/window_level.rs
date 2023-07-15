/// Specifies where a [`Window`] should appear relative to other overlapping windows (on top or under) .
///
/// Levels are groups of windows with respect to their z-position.
///
/// The relative ordering between windows in different window levels is fixed.
/// The z-order of windows within the same window level may change dynamically on user interaction.
///
/// ## Platform-specific
///
/// - **iOS / Android / Web / Wayland:** Unsupported.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLevel {
    /// The window will always be below [`WindowLevel::Normal`] and [`WindowLevel::AlwaysOnTop`] windows.
    ///
    /// This is useful for a widget-based app.
    AlwaysOnBottom,
    /// The default group.
    #[default]
    Normal,
    /// The window will always be on top of [`WindowLevel::Normal`] and [`WindowLevel::AlwaysOnBottom`] windows.
    AlwaysOnTop,
}

impl From<WindowLevel> for winit::window::WindowLevel {
    fn from(value: WindowLevel) -> Self {
        match value {
            WindowLevel::AlwaysOnBottom => Self::AlwaysOnBottom,
            WindowLevel::Normal => Self::Normal,
            WindowLevel::AlwaysOnTop => Self::AlwaysOnTop,
        }
    }
}
