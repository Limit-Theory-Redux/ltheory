/// References a screen monitor.
///
/// Used when centering a [`Window`] on a monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorSelection {
    /// Uses the current monitor of the window.
    ///
    /// If [`WindowPosition::Centered(MonitorSelection::Current)`](WindowPosition::Centered) is used when creating a window,
    /// the window doesn't have a monitor yet, this will fall back to [`WindowPosition::Automatic`].
    Current,
    /// Uses the primary monitor of the system.
    Primary,
    /// Uses the monitor with the specified index.
    Index(usize),
}
