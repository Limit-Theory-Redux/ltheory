/// Cursor data for a [`Window`].
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Cursor {
    /// What the cursor should look like while inside the window.
    pub icon: CursorIcon,

    /// Whether the cursor is visible or not.
    ///
    /// ## Platform-specific
    ///
    /// - **`Windows`**, **`X11`**, and **`Wayland`**: The cursor is hidden only when inside the window.
    /// To stop the cursor from leaving the window, change [`Cursor::grab_mode`] to [`CursorGrabMode::Locked`] or [`CursorGrabMode::Confined`]
    /// - **`macOS`**: The cursor is hidden only when the window is focused.
    /// - **`iOS`** and **`Android`** do not have cursors
    pub visible: bool,

    /// Whether or not the cursor is locked by or confined within the window.
    ///
    /// ## Platform-specific
    ///
    /// - **`Windows`** doesn't support [`CursorGrabMode::Locked`]
    /// - **`macOS`** doesn't support [`CursorGrabMode::Confined`]
    /// - **`iOS/Android`** don't have cursors.
    ///
    /// Since `Windows` and `macOS` have different [`CursorGrabMode`] support, we first try to set the grab mode that was asked for. If it doesn't work then use the alternate grab mode.
    pub grab_mode: CursorGrabMode,

    /// Set whether or not mouse events within *this* window are captured or fall through to the Window below.
    ///
    /// ## Platform-specific
    ///
    /// - iOS / Android / Web / X11: Unsupported.
    pub hit_test: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            icon: CursorIcon::Default,
            visible: true,
            grab_mode: CursorGrabMode::None,
            hit_test: true,
        }
    }
}

/// The icon to display for a [`Window`](crate::window::Window)'s [`Cursor`](crate::window::Cursor).
///
/// Examples of all of these cursors can be found [here](https://www.w3schools.com/cssref/playit.php?filename=playcss_cursor&preval=crosshair).
/// This `enum` is simply a copy of a similar `enum` found in [`winit`](https://docs.rs/winit/latest/winit/window/enum.CursorIcon.html).
/// `winit`, in turn, mostly copied cursor types available in the browser.
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CursorIcon {
    /// The platform-dependent default cursor.
    #[default]
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// An arrow. This is the default cursor on most systems.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done.
    ///
    /// But in contrast with "Wait" the user may still interact with the program.
    /// Often rendered as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,
    /// Cursor showing that something cannot be done.
    NotAllowed,
    /// Indicates that a context menu is available.
    ContextMenu,
    /// Indicates that a cell (or set of cells) may be selected.
    Cell,
    /// Indicates vertical text that may be selected or edited.
    VerticalText,
    /// Indicates that an alias of something is to be created.
    Alias,
    /// Indicates something is to be copied.
    Copy,
    /// Indicates that the dragged item cannot be dropped here.
    NoDrop,
    /// Indicates that something can be grabbed.
    Grab,
    /// Indicates that something is grabbed.
    Grabbing,
    /// Indicates that the user can scroll by dragging the mouse.
    AllScroll,
    /// Indicates that the user can zoom in.
    ZoomIn,
    /// Indicates that the user can zoom out.
    ZoomOut,
    /// Indicates that an edge of a box is to be moved right (east).
    EResize,
    /// Indicates that an edge of a box is to be moved up (north).
    NResize,
    /// Indicates that an edge of a box is to be moved up and right (north/east).
    NeResize,
    /// indicates that an edge of a box is to be moved up and left (north/west).
    NwResize,
    /// Indicates that an edge of a box is to be moved down (south).
    SResize,
    /// The cursor indicates that an edge of a box is to be moved down and right (south/east).
    SeResize,
    /// The cursor indicates that an edge of a box is to be moved down and left (south/west).
    SwResize,
    /// Indicates that an edge of a box is to be moved left (west).
    WResize,
    /// Indicates a bidirectional resize cursor.
    EwResize,
    /// Indicates a bidirectional resize cursor.
    NsResize,
    /// Indicates a bidirectional resize cursor.
    NeswResize,
    /// Indicates a bidirectional resize cursor.
    NwseResize,
    /// Indicates that a column can be resized horizontally.
    ColResize,
    /// Indicates that the row can be resized vertically.
    RowResize,
}

/// Defines if and how the [`Cursor`] is grabbed by a [`Window`].
///
/// ## Platform-specific
///
/// - **`Windows`** doesn't support [`CursorGrabMode::Locked`]
/// - **`macOS`** doesn't support [`CursorGrabMode::Confined`]
/// - **`iOS/Android`** don't have cursors.
///
/// Since `Windows` and `macOS` have different [`CursorGrabMode`] support, we first try to set the grab mode that was asked for. If it doesn't work then use the alternate grab mode.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorGrabMode {
    /// The cursor can freely leave the window.
    #[default]
    None,
    /// The cursor is confined to the window area.
    Confined,
    /// The cursor is locked inside the window area to a certain position.
    Locked,
}
