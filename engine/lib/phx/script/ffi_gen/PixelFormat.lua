-- AUTO GENERATED. DO NOT MODIFY!
-- PixelFormat -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'PixelFormat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PixelFormat

    do -- C Definitions
        ffi.cdef [[
            int PixelFormat_Components (PixelFormat this);
        ]]
    end

    do -- Global Symbol Table
        PixelFormat = {
            Components = libphx.PixelFormat_Components,
        }

        if onDef_PixelFormat then onDef_PixelFormat(PixelFormat, mt) end
        PixelFormat = setmetatable(PixelFormat, mt)
    end

    return PixelFormat
end

return Loader
