-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr      BlendMode_ToString(BlendMode);
        ]]
    end

    do -- Global Symbol Table
        BlendMode = {
            Disabled     = 0,
            Additive     = 1,
            Alpha        = 2,
            PreMultAlpha = 3,

            ToString     = libphx.BlendMode_ToString,
        }

        if onDef_BlendMode then onDef_BlendMode(BlendMode, mt) end
        BlendMode = setmetatable(BlendMode, mt)
    end

    return BlendMode
end

return Loader
