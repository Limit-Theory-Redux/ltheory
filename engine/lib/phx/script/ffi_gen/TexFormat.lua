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
            TexFormat TexFormat_R8;
            TexFormat TexFormat_R16;
            TexFormat TexFormat_R16F;
            TexFormat TexFormat_R32F;
            TexFormat TexFormat_RG8;
            TexFormat TexFormat_RG16;
            TexFormat TexFormat_RG16F;
            TexFormat TexFormat_RG32F;
            TexFormat TexFormat_RGB8;
            TexFormat TexFormat_RGBA8;
            TexFormat TexFormat_RGBA16;
            TexFormat TexFormat_RGBA16F;
            TexFormat TexFormat_RGBA32F;
            TexFormat TexFormat_Depth16;
            TexFormat TexFormat_Depth24;
            TexFormat TexFormat_Depth32F;

            cstr      TexFormat_ToString(TexFormat);

            int  TexFormat_Components (TexFormat this);
            int  TexFormat_GetSize    (TexFormat this);
            bool TexFormat_IsColor    (TexFormat this);
            bool TexFormat_IsDepth    (TexFormat this);
        ]]
    end

    do -- Global Symbol Table
        TexFormat = {
            R8       = libphx.TexFormat_R8,
            R16      = libphx.TexFormat_R16,
            R16F     = libphx.TexFormat_R16F,
            R32F     = libphx.TexFormat_R32F,
            RG8      = libphx.TexFormat_RG8,
            RG16     = libphx.TexFormat_RG16,
            RG16F    = libphx.TexFormat_RG16F,
            RG32F    = libphx.TexFormat_RG32F,
            RGB8     = libphx.TexFormat_RGB8,
            RGBA8    = libphx.TexFormat_RGBA8,
            RGBA16   = libphx.TexFormat_RGBA16,
            RGBA16F  = libphx.TexFormat_RGBA16F,
            RGBA32F  = libphx.TexFormat_RGBA32F,
            Depth16  = libphx.TexFormat_Depth16,
            Depth24  = libphx.TexFormat_Depth24,
            Depth32F = libphx.TexFormat_Depth32F,

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
