-- WindowMode ------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local WindowMode

do -- C Definitions
    ffi.cdef [[
        WindowMode WindowMode_Windowed;
        WindowMode WindowMode_BorderlessFullscreen;
        WindowMode WindowMode_SizedFullscreen;
        WindowMode WindowMode_Fullscreen;

        cstr       WindowMode_ToString(WindowMode);
    ]]
end

do -- Global Symbol Table
    WindowMode = {
        Windowed             = libphx.WindowMode_Windowed,
        BorderlessFullscreen = libphx.WindowMode_BorderlessFullscreen,
        SizedFullscreen      = libphx.WindowMode_SizedFullscreen,
        Fullscreen           = libphx.WindowMode_Fullscreen,

        ToString             = libphx.WindowMode_ToString,
    }

    if onDef_WindowMode then onDef_WindowMode(WindowMode, mt) end
    WindowMode = setmetatable(WindowMode, mt)
end

return WindowMode