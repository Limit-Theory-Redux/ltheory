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
            PixelFormat PixelFormat_DepthComponent;
            PixelFormat PixelFormat_Red;
            PixelFormat PixelFormat_RGB;
            PixelFormat PixelFormat_RGBA;
            PixelFormat PixelFormat_BGR;
            PixelFormat PixelFormat_BGRA;
            PixelFormat PixelFormat_RG;

            cstr        PixelFormat_ToString(PixelFormat);

            int PixelFormat_Components (PixelFormat this);
        ]]
    end

    do -- Global Symbol Table
        PixelFormat = {
            DepthComponent = libphx.PixelFormat_DepthComponent,
            Red            = libphx.PixelFormat_Red,
            RGB            = libphx.PixelFormat_RGB,
            RGBA           = libphx.PixelFormat_RGBA,
            BGR            = libphx.PixelFormat_BGR,
            BGRA           = libphx.PixelFormat_BGRA,
            RG             = libphx.PixelFormat_RG,

            ToString       = libphx.PixelFormat_ToString,

            Components = libphx.PixelFormat_Components,
        }

        if onDef_PixelFormat then onDef_PixelFormat(PixelFormat, mt) end
        PixelFormat = setmetatable(PixelFormat, mt)
    end

    return PixelFormat
end

return Loader
