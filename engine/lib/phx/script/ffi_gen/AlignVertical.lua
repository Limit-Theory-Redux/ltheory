-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr          AlignVertical_ToString(AlignVertical);
        ]]
    end

    do -- Global Symbol Table
        AlignVertical = {
            Default  = 0,
            Center   = 1,
            Top      = 2,
            Bottom   = 3,
            Expand   = 4,
            Stretch  = 5,

            ToString = libphx.AlignVertical_ToString,
        }

        if onDef_AlignVertical then onDef_AlignVertical(AlignVertical, mt) end
        AlignVertical = setmetatable(AlignVertical, mt)
    end

    return AlignVertical
end

return Loader
