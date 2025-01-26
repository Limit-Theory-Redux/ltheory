-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- The icon to display for a [`Window`](crate::window::Window)'s [`Cursor`](crate::window::Cursor).
-- 
-- Examples of all of these cursors can be found [here](https://www.w3schools.com/cssref/playit.php?filename=playcss_cursor&preval=crosshair).
-- This `enum` is simply a copy of a similar `enum` found in [`winit`](https://docs.rs/winit/latest/winit/window/enum.CursorIcon.html).
-- `winit`, in turn, mostly copied cursor types available in the browser.
---@class CursorIcon
---@field Default integer The platform-dependent default cursor.
---@field Crosshair integer A simple crosshair.
---@field Pointer integer A hand (often used to indicate links in web browsers).
---@field Move integer Indicates something is to be moved.
---@field Text integer Indicates text that may be selected or edited.
---@field Wait integer Program busy indicator.
---@field Help integer Help indicator (often rendered as a "?")
---@field Progress integer Progress indicator. Shows that processing is being done.  But in contrast with "Wait" the user may still interact with the program. Often rendered as a spinning beach ball, or an arrow with a watch or hourglass.
---@field NotAllowed integer Cursor showing that something cannot be done.
---@field ContextMenu integer Indicates that a context menu is available.
---@field Cell integer Indicates that a cell (or set of cells) may be selected.
---@field VerticalText integer Indicates vertical text that may be selected or edited.
---@field Alias integer Indicates that an alias of something is to be created.
---@field Copy integer Indicates something is to be copied.
---@field NoDrop integer Indicates that the dragged item cannot be dropped here.
---@field Grab integer Indicates that something can be grabbed.
---@field Grabbing integer Indicates that something is grabbed.
---@field AllScroll integer Indicates that the user can scroll by dragging the mouse.
---@field ZoomIn integer Indicates that the user can zoom in.
---@field ZoomOut integer Indicates that the user can zoom out.
---@field EResize integer Indicates that an edge of a box is to be moved right (east).
---@field NResize integer Indicates that an edge of a box is to be moved up (north).
---@field NeResize integer Indicates that an edge of a box is to be moved up and right (north/east).
---@field NwResize integer indicates that an edge of a box is to be moved up and left (north/west).
---@field SResize integer Indicates that an edge of a box is to be moved down (south).
---@field SeResize integer The cursor indicates that an edge of a box is to be moved down and right (south/east).
---@field SwResize integer The cursor indicates that an edge of a box is to be moved down and left (south/west).
---@field WResize integer Indicates that an edge of a box is to be moved left (west).
---@field EwResize integer Indicates a bidirectional resize cursor.
---@field NsResize integer Indicates a bidirectional resize cursor.
---@field NeswResize integer Indicates a bidirectional resize cursor.
---@field NwseResize integer Indicates a bidirectional resize cursor.
---@field ColResize integer Indicates that a column can be resized horizontally.
---@field RowResize integer Indicates that the row can be resized vertically.
CursorIcon = {
    -- The platform-dependent default cursor.
    Default = 0,
    -- A simple crosshair.
    Crosshair = 1,
    -- A hand (often used to indicate links in web browsers).
    Pointer = 2,
    -- Indicates something is to be moved.
    Move = 3,
    -- Indicates text that may be selected or edited.
    Text = 4,
    -- Program busy indicator.
    Wait = 5,
    -- Help indicator (often rendered as a "?")
    Help = 6,
    -- Progress indicator. Shows that processing is being done.
    -- 
    -- But in contrast with "Wait" the user may still interact with the program.
    -- Often rendered as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress = 7,
    -- Cursor showing that something cannot be done.
    NotAllowed = 8,
    -- Indicates that a context menu is available.
    ContextMenu = 9,
    -- Indicates that a cell (or set of cells) may be selected.
    Cell = 10,
    -- Indicates vertical text that may be selected or edited.
    VerticalText = 11,
    -- Indicates that an alias of something is to be created.
    Alias = 12,
    -- Indicates something is to be copied.
    Copy = 13,
    -- Indicates that the dragged item cannot be dropped here.
    NoDrop = 14,
    -- Indicates that something can be grabbed.
    Grab = 15,
    -- Indicates that something is grabbed.
    Grabbing = 16,
    -- Indicates that the user can scroll by dragging the mouse.
    AllScroll = 17,
    -- Indicates that the user can zoom in.
    ZoomIn = 18,
    -- Indicates that the user can zoom out.
    ZoomOut = 19,
    -- Indicates that an edge of a box is to be moved right (east).
    EResize = 20,
    -- Indicates that an edge of a box is to be moved up (north).
    NResize = 21,
    -- Indicates that an edge of a box is to be moved up and right (north/east).
    NeResize = 22,
    -- indicates that an edge of a box is to be moved up and left (north/west).
    NwResize = 23,
    -- Indicates that an edge of a box is to be moved down (south).
    SResize = 24,
    -- The cursor indicates that an edge of a box is to be moved down and right (south/east).
    SeResize = 25,
    -- The cursor indicates that an edge of a box is to be moved down and left (south/west).
    SwResize = 26,
    -- Indicates that an edge of a box is to be moved left (west).
    WResize = 27,
    -- Indicates a bidirectional resize cursor.
    EwResize = 28,
    -- Indicates a bidirectional resize cursor.
    NsResize = 29,
    -- Indicates a bidirectional resize cursor.
    NeswResize = 30,
    -- Indicates a bidirectional resize cursor.
    NwseResize = 31,
    -- Indicates that a column can be resized horizontally.
    ColResize = 32,
    -- Indicates that the row can be resized vertically.
    RowResize = 33,
}

