---@meta

-- The presentation mode specifies when a frame is presented to the window.
-- 
-- `Vsync` will cap the framerate by the display refresh rate, while `NoVsync` will present as fast as possible.
---@class PresentMode
---@field Vsync integer 
---@field NoVsync integer 
PresentMode = {
    Vsync = 0,
    NoVsync = 1,
}

