-- AUTO GENERATED. DO NOT MODIFY!
-- TexWrapMode -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 TexWrapMode;
    ]]

    return 2, 'TexWrapMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexWrapMode

    do -- C Definitions
        ffi.cdef [[
            cstr        TexWrapMode_ToString(TexWrapMode);
        ]]
    end

    do -- Global Symbol Table
        TexWrapMode = {
            Clamp        = 33071,
            MirrorClamp  = 34627,
            MirrorRepeat = 33648,
            Repeat       = 10497,

            ToString     = libphx.TexWrapMode_ToString,
        }

        if onDef_TexWrapMode then onDef_TexWrapMode(TexWrapMode, mt) end
        TexWrapMode = setmetatable(TexWrapMode, mt)
    end

    return TexWrapMode
end

return Loader
