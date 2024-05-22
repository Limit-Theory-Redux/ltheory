---@meta

-- Defines the way a [`Window`] is displayed.
---@enum WindowMode
WindowMode = {
    -- The window should take a portion of the screen, using the window resolution size.
    Windowed = 0,
    -- The window should appear fullscreen by being borderless and using the full
    -- size of the screen.
    -- 
    -- When setting this, the window's physical size will be modified to match the size
    -- of the current monitor resolution, and the logical size will follow based
    -- on the scale factor, see [`WindowResolution`].
    BorderlessFullscreen = 1,
    -- The window should be in "true"/"legacy" Fullscreen mode.
    -- 
    -- When setting this, the operating system will be requested to use the
    -- **closest** resolution available for the current monitor to match as
    -- closely as possible the window's physical size.
    -- After that, the window's physical size will be modified to match
    -- that monitor resolution, and the logical size will follow based on the
    -- scale factor, see [`WindowResolution`].
    SizedFullscreen = 2,
    -- The window should be in "true"/"legacy" Fullscreen mode.
    -- 
    -- When setting this, the operating system will be requested to use the
    -- **biggest** resolution available for the current monitor.
    -- After that, the window's physical size will be modified to match
    -- that monitor resolution, and the logical size will follow based on the
    -- scale factor, see [`WindowResolution`].
    Fullscreen = 3,
}

