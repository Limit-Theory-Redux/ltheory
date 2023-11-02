-- DockingFlag -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 DockingFlag;
    ]]

    return 2, 'DockingFlag'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local DockingFlag

    do -- C Definitions
        ffi.cdef [[
            DockingFlag DockingFlag_None;
            DockingFlag DockingFlag_Left;
            DockingFlag DockingFlag_Right;
            DockingFlag DockingFlag_Top;
            DockingFlag DockingFlag_Bottom;

            cstr        DockingFlag_ToString(DockingFlag);
        ]]
    end

    do -- Global Symbol Table
        DockingFlag = {
            None     = libphx.DockingFlag_None,
            Left     = libphx.DockingFlag_Left,
            Right    = libphx.DockingFlag_Right,
            Top      = libphx.DockingFlag_Top,
            Bottom   = libphx.DockingFlag_Bottom,

            ToString = libphx.DockingFlag_ToString,
        }

        if onDef_DockingFlag then onDef_DockingFlag(DockingFlag, mt) end
        DockingFlag = setmetatable(DockingFlag, mt)
    end

    return DockingFlag
end

return Loader
