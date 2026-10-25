-- AUTO GENERATED. DO NOT MODIFY!
-- PixelFormat -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 PixelFormat;
    ]]

    return 2, 'PixelFormat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PixelFormat

    do -- C Definitions
        ffi.cdef [[
            cstr        PixelFormat_ToString(PixelFormat);

            int PixelFormat_Components (PixelFormat this);
        ]]
    end

    do -- Global Symbol Table
        PixelFormat = {
            DepthComponent = 0,
            Red            = 1,
            RGB            = 2,
            RGBA           = 3,
            BGR            = 4,
            BGRA           = 5,
            RG             = 6,

            ToString       = libphx.PixelFormat_ToString,

            Components = libphx.PixelFormat_Components,
        }

        if onDef_PixelFormat then onDef_PixelFormat(PixelFormat, mt) end
        PixelFormat = setmetatable(PixelFormat, mt)
    end

    return PixelFormat
end

return Loader
