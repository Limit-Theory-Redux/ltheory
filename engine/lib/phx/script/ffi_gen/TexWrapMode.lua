-- AUTO GENERATED. DO NOT MODIFY!
-- TexWrapMode -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 TexWrapMode;
    ]]

    return 2, 'TexWrapMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexWrapMode

    do -- C Definitions
        ffi.cdef [[
            TexWrapMode TexWrapMode_Clamp;
            TexWrapMode TexWrapMode_MirrorClamp;
            TexWrapMode TexWrapMode_MirrorRepeat;
            TexWrapMode TexWrapMode_Repeat;

            cstr        TexWrapMode_ToString(TexWrapMode);
        ]]
    end

    do -- Global Symbol Table
        TexWrapMode = {
            Clamp        = libphx.TexWrapMode_Clamp,
            MirrorClamp  = libphx.TexWrapMode_MirrorClamp,
            MirrorRepeat = libphx.TexWrapMode_MirrorRepeat,
            Repeat       = libphx.TexWrapMode_Repeat,

            ToString     = libphx.TexWrapMode_ToString,
        }

        if onDef_TexWrapMode then onDef_TexWrapMode(TexWrapMode, mt) end
        TexWrapMode = setmetatable(TexWrapMode, mt)
    end

    return TexWrapMode
end

return Loader
