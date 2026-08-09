-- AUTO GENERATED. DO NOT MODIFY!
-- TexFormat -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'TexFormat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexFormat

    do -- C Definitions
        ffi.cdef [[
            int  TexFormat_Components (TexFormat this);
            int  TexFormat_GetSize    (TexFormat this);
            bool TexFormat_IsColor    (TexFormat this);
            bool TexFormat_IsDepth    (TexFormat this);
        ]]
    end

    do -- Global Symbol Table
        TexFormat = {
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
