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
            PresentMode PresentMode_Vsync;
            PresentMode PresentMode_NoVsync;

            cstr        PresentMode_ToString(PresentMode);
        ]]
    end

    do -- Global Symbol Table
        PresentMode = {
            Vsync    = libphx.PresentMode_Vsync,
            NoVsync  = libphx.PresentMode_NoVsync,

            ToString = libphx.PresentMode_ToString,
        }

        if onDef_PresentMode then onDef_PresentMode(PresentMode, mt) end
        PresentMode = setmetatable(PresentMode, mt)
    end

    return PresentMode
end

return Loader
