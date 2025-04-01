-- AUTO GENERATED. DO NOT MODIFY!
-- PresentMode -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 PresentMode;
    ]]

    return 2, 'PresentMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PresentMode

    do -- C Definitions
        ffi.cdef [[
            cstr        PresentMode_ToString(PresentMode);
        ]]
    end

    do -- Global Symbol Table
        PresentMode = {
            Vsync    = 0,
            NoVsync  = 1,

            ToString = libphx.PresentMode_ToString,
        }

        if onDef_PresentMode then onDef_PresentMode(PresentMode, mt) end
        PresentMode = setmetatable(PresentMode, mt)
    end

    return PresentMode
end

return Loader
