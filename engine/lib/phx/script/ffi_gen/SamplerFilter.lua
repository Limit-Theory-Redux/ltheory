-- AUTO GENERATED. DO NOT MODIFY!
-- SamplerFilter ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 SamplerFilter;
    ]]

    return 2, 'SamplerFilter'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SamplerFilter

    do -- C Definitions
        ffi.cdef [[
            cstr          SamplerFilter_ToString(SamplerFilter);
        ]]
    end

    do -- Global Symbol Table
        SamplerFilter = {
            Point    = 0,
            Linear   = 1,

            ToString = libphx.SamplerFilter_ToString,
        }

        if onDef_SamplerFilter then onDef_SamplerFilter(SamplerFilter, mt) end
        SamplerFilter = setmetatable(SamplerFilter, mt)
    end

    return SamplerFilter
end

return Loader
