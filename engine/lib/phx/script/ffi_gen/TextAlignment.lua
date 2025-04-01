-- AUTO GENERATED. DO NOT MODIFY!
-- TextAlignment ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 TextAlignment;
    ]]

    return 2, 'TextAlignment'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TextAlignment

    do -- C Definitions
        ffi.cdef [[
            cstr          TextAlignment_ToString(TextAlignment);
        ]]
    end

    do -- Global Symbol Table
        TextAlignment = {
            Start     = 0,
            Middle    = 1,
            End       = 2,
            Justified = 3,

            ToString  = libphx.TextAlignment_ToString,
        }

        if onDef_TextAlignment then onDef_TextAlignment(TextAlignment, mt) end
        TextAlignment = setmetatable(TextAlignment, mt)
    end

    return TextAlignment
end

return Loader
