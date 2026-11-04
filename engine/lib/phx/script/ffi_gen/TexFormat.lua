-- AUTO GENERATED. DO NOT MODIFY!
-- TexFormat -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 TexFormat;
    ]]

    return 2, 'TexFormat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexFormat

    do -- C Definitions
        ffi.cdef [[
            cstr      TexFormat_ToString(TexFormat);

            int  TexFormat_Components (TexFormat this);
            int  TexFormat_GetSize    (TexFormat this);
            bool TexFormat_IsColor    (TexFormat this);
            bool TexFormat_IsDepth    (TexFormat this);
        ]]
    end

    do -- Global Symbol Table
        TexFormat = {
            R8       = 0,
            R16      = 1,
            R16F     = 2,
            R32F     = 3,
            RG8      = 4,
            RG16     = 5,
            RG16F    = 6,
            RG32F    = 7,
            RGBA8    = 8,
            RGBA16   = 9,
            RGBA16F  = 10,
            RGBA32F  = 11,
            Depth16  = 12,
            Depth24  = 13,
            Depth32F = 14,

            ToString = libphx.TexFormat_ToString,

            Components = libphx.TexFormat_Components,
            GetSize    = libphx.TexFormat_GetSize,
            IsColor    = libphx.TexFormat_IsColor,
            IsDepth    = libphx.TexFormat_IsDepth,
        }

        if onDef_TexFormat then onDef_TexFormat(TexFormat, mt) end
        TexFormat = setmetatable(TexFormat, mt)
    end

    return TexFormat
end

return Loader
