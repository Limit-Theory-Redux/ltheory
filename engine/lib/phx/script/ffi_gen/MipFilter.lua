-- AUTO GENERATED. DO NOT MODIFY!
-- MipFilter -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 MipFilter;
    ]]

    return 2, 'MipFilter'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MipFilter

    do -- C Definitions
        ffi.cdef [[
            cstr      MipFilter_ToString(MipFilter);
        ]]
    end

    do -- Global Symbol Table
        MipFilter = {
            None     = 0,
            Point    = 1,
            Linear   = 2,

            ToString = libphx.MipFilter_ToString,
        }

        if onDef_MipFilter then onDef_MipFilter(MipFilter, mt) end
        MipFilter = setmetatable(MipFilter, mt)
    end

    return MipFilter
end

return Loader
