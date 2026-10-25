-- AUTO GENERATED. DO NOT MODIFY!
-- Samplers --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 Samplers;
    ]]

    return 2, 'Samplers'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Samplers

    do -- C Definitions
        ffi.cdef [[
            cstr     Samplers_ToString(Samplers);
        ]]
    end

    do -- Global Symbol Table
        Samplers = {
            Point                = 0,
            PointRepeat          = 1,
            LinearClamp          = 2,
            LinearRepeat         = 3,
            LinearMipClamp       = 4,
            LinearMipRepeat      = 5,
            LinearMipRepeatAniso = 6,

            ToString             = libphx.Samplers_ToString,
        }

        if onDef_Samplers then onDef_Samplers(Samplers, mt) end
        Samplers = setmetatable(Samplers, mt)
    end

    return Samplers
end

return Loader
