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
            TexFilter TexFilter_Point;
            TexFilter TexFilter_PointMipPoint;
            TexFilter TexFilter_PointMipLinear;
            TexFilter TexFilter_Linear;
            TexFilter TexFilter_LinearMipPoint;
            TexFilter TexFilter_LinearMipLinear;

            cstr      TexFilter_ToString(TexFilter);
        ]]
    end

    do -- Global Symbol Table
        TexFilter = {
            Point           = libphx.TexFilter_Point,
            PointMipPoint   = libphx.TexFilter_PointMipPoint,
            PointMipLinear  = libphx.TexFilter_PointMipLinear,
            Linear          = libphx.TexFilter_Linear,
            LinearMipPoint  = libphx.TexFilter_LinearMipPoint,
            LinearMipLinear = libphx.TexFilter_LinearMipLinear,

            ToString        = libphx.TexFilter_ToString,
        }

        if onDef_TexFilter then onDef_TexFilter(TexFilter, mt) end
        TexFilter = setmetatable(TexFilter, mt)
    end

    return TexFilter
end

return Loader
