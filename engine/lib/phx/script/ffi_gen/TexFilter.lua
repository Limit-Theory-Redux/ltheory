-- AUTO GENERATED. DO NOT MODIFY!
-- TexFilter -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 TexFilter;
    ]]

    return 2, 'TexFilter'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexFilter

    do -- C Definitions
        ffi.cdef [[
            cstr      TexFilter_ToString(TexFilter);
        ]]
    end

    do -- Global Symbol Table
        TexFilter = {
            Point           = 9728,
            PointMipPoint   = 9984,
            PointMipLinear  = 9986,
            Linear          = 9729,
            LinearMipPoint  = 9985,
            LinearMipLinear = 9987,

            ToString        = libphx.TexFilter_ToString,
        }

        if onDef_TexFilter then onDef_TexFilter(TexFilter, mt) end
        TexFilter = setmetatable(TexFilter, mt)
    end

    return TexFilter
end

return Loader
