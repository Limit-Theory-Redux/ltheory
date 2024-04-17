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
            PresentMode PresentMode_AutoVsync;
            PresentMode PresentMode_AutoNoVsync;
            PresentMode PresentMode_Immediate;
            PresentMode PresentMode_Mailbox;
            PresentMode PresentMode_Fifo;

            cstr        PresentMode_ToString(PresentMode);
        ]]
    end

    do -- Global Symbol Table
        PresentMode = {
            AutoVsync   = libphx.PresentMode_AutoVsync,
            AutoNoVsync = libphx.PresentMode_AutoNoVsync,
            Immediate   = libphx.PresentMode_Immediate,
            Mailbox     = libphx.PresentMode_Mailbox,
            Fifo        = libphx.PresentMode_Fifo,

            ToString    = libphx.PresentMode_ToString,
        }

        if onDef_PresentMode then onDef_PresentMode(PresentMode, mt) end
        PresentMode = setmetatable(PresentMode, mt)
    end

    return PresentMode
end

return Loader
