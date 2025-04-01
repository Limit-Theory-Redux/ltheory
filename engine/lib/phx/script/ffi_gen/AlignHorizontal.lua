-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr            AlignHorizontal_ToString(AlignHorizontal);
        ]]
    end

    do -- Global Symbol Table
        AlignHorizontal = {
            Default  = 0,
            Center   = 1,
            Left     = 2,
            Right    = 3,
            Expand   = 4,
            Stretch  = 5,

            ToString = libphx.AlignHorizontal_ToString,
        }

        if onDef_AlignHorizontal then onDef_AlignHorizontal(AlignHorizontal, mt) end
        AlignHorizontal = setmetatable(AlignHorizontal, mt)
    end

    return AlignHorizontal
end

return Loader
