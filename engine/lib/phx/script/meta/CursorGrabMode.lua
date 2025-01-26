-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Defines if and how the [`Cursor`] is grabbed by a [`Window`].
-- 
-- ## Platform-specific
-- 
-- - **`Windows`** doesn't support [`CursorGrabMode::Locked`]
-- - **`macOS`** doesn't support [`CursorGrabMode::Confined`]
-- - **`iOS/Android`** don't have cursors.
-- 
-- Since `Windows` and `macOS` have different [`CursorGrabMode`] support, we first try to set the grab mode that was asked for. If it doesn't work then use the alternate grab mode.
---@class CursorGrabMode
---@field None integer The cursor can freely leave the window.
---@field Confined integer The cursor is confined to the window area.
---@field Locked integer The cursor is locked inside the window area to a certain position.
CursorGrabMode = {
    -- The cursor can freely leave the window.
    None = 0,
    -- The cursor is confined to the window area.
    Confined = 1,
    -- The cursor is locked inside the window area to a certain position.
    Locked = 2,
}

