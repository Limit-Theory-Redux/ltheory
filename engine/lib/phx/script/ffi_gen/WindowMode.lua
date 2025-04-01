-- AUTO GENERATED. DO NOT MODIFY!
-- WindowMode ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 WindowMode;
    ]]

    return 2, 'WindowMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local WindowMode

    do -- C Definitions
        ffi.cdef [[
            cstr       WindowMode_ToString(WindowMode);
        ]]
    end

    do -- Global Symbol Table
        WindowMode = {
            Windowed             = 0,
            BorderlessFullscreen = 1,
            SizedFullscreen      = 2,
            Fullscreen           = 3,

            ToString             = libphx.WindowMode_ToString,
        }

        if onDef_WindowMode then onDef_WindowMode(WindowMode, mt) end
        WindowMode = setmetatable(WindowMode, mt)
    end

    return WindowMode
end

return Loader
