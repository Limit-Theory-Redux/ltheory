-- AUTO GENERATED. DO NOT MODIFY!
-- TexFilter -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 TexFilter;
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
            Point           = 0,
            PointMipPoint   = 1,
            PointMipLinear  = 2,
            Linear          = 3,
            LinearMipPoint  = 4,
            LinearMipLinear = 5,

            ToString        = libphx.TexFilter_ToString,
        }

        if onDef_TexFilter then onDef_TexFilter(TexFilter, mt) end
        TexFilter = setmetatable(TexFilter, mt)
    end

    return TexFilter
end

return Loader
