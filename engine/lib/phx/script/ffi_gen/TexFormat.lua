-- AUTO GENERATED. DO NOT MODIFY!
-- TexFormat -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 TexFormat;
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
            bool TexFormat_IsValid    (TexFormat this);
        ]]
    end

    do -- Global Symbol Table
        TexFormat = {
            Undefined = 0,
            R8        = 33321,
            R16       = 33322,
            R16F      = 33325,
            R32F      = 33326,
            RG8       = 6407,
            RG16      = 33324,
            RG16F     = 33327,
            RG32F     = 33328,
            RGB8      = 32849,
            RGBA8     = 32856,
            RGBA16    = 32859,
            RGBA16F   = 34842,
            RGBA32F   = 34836,
            Depth16   = 33189,
            Depth24   = 33190,
            Depth32F  = 36012,

            ToString  = libphx.TexFormat_ToString,

            Components = libphx.TexFormat_Components,
            GetSize    = libphx.TexFormat_GetSize,
            IsColor    = libphx.TexFormat_IsColor,
            IsDepth    = libphx.TexFormat_IsDepth,
            IsValid    = libphx.TexFormat_IsValid,
        }

        if onDef_TexFormat then onDef_TexFormat(TexFormat, mt) end
        TexFormat = setmetatable(TexFormat, mt)
    end

    return TexFormat
end

return Loader
