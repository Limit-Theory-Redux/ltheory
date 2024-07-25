-- BlendMode -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 BlendMode;
    ]]

    return 2, 'BlendMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BlendMode

    do -- C Definitions
        ffi.cdef [[
            BlendMode BlendMode_Disabled;
            BlendMode BlendMode_Additive;
            BlendMode BlendMode_Alpha;
            BlendMode BlendMode_PreMultAlpha;

            cstr      BlendMode_ToString(BlendMode);
        ]]
    end

    do -- Global Symbol Table
        BlendMode = {
            Disabled     = libphx.BlendMode_Disabled,
            Additive     = libphx.BlendMode_Additive,
            Alpha        = libphx.BlendMode_Alpha,
            PreMultAlpha = libphx.BlendMode_PreMultAlpha,

            ToString     = libphx.BlendMode_ToString,
        }

        if onDef_BlendMode then onDef_BlendMode(BlendMode, mt) end
        BlendMode = setmetatable(BlendMode, mt)
    end

    return BlendMode
end

return Loader
