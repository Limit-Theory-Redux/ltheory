-- Docking ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 Docking;
    ]]

    return 2, 'Docking'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Docking

    do -- C Definitions
        ffi.cdef [[
            Docking Docking_None;
            Docking Docking_Left;
            Docking Docking_Right;
            Docking Docking_Top;
            Docking Docking_Bottom;
            Docking Docking_StretchHorizontal;
            Docking Docking_StretchVertical;
            Docking Docking_StretchAll;

            cstr    Docking_ToString(Docking);
        ]]
    end

    do -- Global Symbol Table
        Docking = {
            None              = libphx.Docking_None,
            Left              = libphx.Docking_Left,
            Right             = libphx.Docking_Right,
            Top               = libphx.Docking_Top,
            Bottom            = libphx.Docking_Bottom,
            StretchHorizontal = libphx.Docking_StretchHorizontal,
            StretchVertical   = libphx.Docking_StretchVertical,
            StretchAll        = libphx.Docking_StretchAll,

            ToString          = libphx.Docking_ToString,
        }

        if onDef_Docking then onDef_Docking(Docking, mt) end
        Docking = setmetatable(Docking, mt)
    end

    return Docking
end

return Loader
