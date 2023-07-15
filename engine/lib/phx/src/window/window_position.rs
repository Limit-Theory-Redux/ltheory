use glam::IVec2;

use super::MonitorSelection;

/// Defines where a [`Window`] should be placed on the screen.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum WindowPosition {
    /// Position will be set by the window manager.
    /// Bevy will delegate this decision to the window manager and no guarantees can be made about where the window will be placed.
    ///
    /// Used at creation but will be changed to [`At`](WindowPosition::At).
    #[default]
    Automatic,
    /// Window will be centered on the selected monitor.
    ///
    /// Note that this does not account for window decorations.
    ///
    /// Used at creation or for update but will be changed to [`At`](WindowPosition::At)
    Centered(MonitorSelection),
    /// The window's top-left corner should be placed at the specified position (in physical pixels).
    ///
    /// (0,0) represents top-left corner of screen space.
    At(IVec2),
}

impl WindowPosition {
    /// Creates a new [`WindowPosition`] at a position.
    pub fn new(position: IVec2) -> Self {
        Self::At(position)
    }

    /// Set the position to a specific point.
    pub fn set(&mut self, position: IVec2) {
        *self = WindowPosition::At(position);
    }

    /// Set the window to a specific monitor.
    pub fn center(&mut self, monitor: MonitorSelection) {
        *self = WindowPosition::Centered(monitor);
    }
}
