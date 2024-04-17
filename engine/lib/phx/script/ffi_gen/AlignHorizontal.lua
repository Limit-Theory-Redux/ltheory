-- AlignHorizontal -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 AlignHorizontal;
    ]]

    return 2, 'AlignHorizontal'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local AlignHorizontal

    do -- C Definitions
        ffi.cdef [[
            AlignHorizontal AlignHorizontal_Default;
            AlignHorizontal AlignHorizontal_Center;
            AlignHorizontal AlignHorizontal_Left;
            AlignHorizontal AlignHorizontal_Right;
            AlignHorizontal AlignHorizontal_Stretch;

            cstr            AlignHorizontal_ToString(AlignHorizontal);
        ]]
    end

    do -- Global Symbol Table
        AlignHorizontal = {
            Default  = libphx.AlignHorizontal_Default,
            Center   = libphx.AlignHorizontal_Center,
            Left     = libphx.AlignHorizontal_Left,
            Right    = libphx.AlignHorizontal_Right,
            Stretch  = libphx.AlignHorizontal_Stretch,

            ToString = libphx.AlignHorizontal_ToString,
        }

        if onDef_AlignHorizontal then onDef_AlignHorizontal(AlignHorizontal, mt) end
        AlignHorizontal = setmetatable(AlignHorizontal, mt)
    end

    return AlignHorizontal
end

return Loader
