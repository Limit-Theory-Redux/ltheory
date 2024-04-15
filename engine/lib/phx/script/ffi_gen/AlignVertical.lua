-- AlignVertical ---------------------------------------------------------------


local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 AlignVertical;
    ]]

    return 2, 'AlignVertical'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local AlignVertical

    do -- C Definitions
        ffi.cdef [[
            AlignVertical AlignVertical_Default;
            AlignVertical AlignVertical_Center;
            AlignVertical AlignVertical_Top;
            AlignVertical AlignVertical_Bottom;
            AlignVertical AlignVertical_Stretch;

            cstr          AlignVertical_ToString(AlignVertical);
        ]]
    end

    do -- Global Symbol Table
        AlignVertical = {
            Default  = libphx.AlignVertical_Default,
            Center   = libphx.AlignVertical_Center,
            Top      = libphx.AlignVertical_Top,
            Bottom   = libphx.AlignVertical_Bottom,
            Stretch  = libphx.AlignVertical_Stretch,

            ToString = libphx.AlignVertical_ToString,
        }

        if onDef_AlignVertical then onDef_AlignVertical(AlignVertical, mt) end
        AlignVertical = setmetatable(AlignVertical, mt)
    end

    return AlignVertical
end

return Loader
