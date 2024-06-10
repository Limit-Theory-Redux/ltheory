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
            TextAlignment TextAlignment_Start;
            TextAlignment TextAlignment_Middle;
            TextAlignment TextAlignment_End;
            TextAlignment TextAlignment_Justified;

            cstr          TextAlignment_ToString(TextAlignment);
        ]]
    end

    do -- Global Symbol Table
        TextAlignment = {
            Start     = libphx.TextAlignment_Start,
            Middle    = libphx.TextAlignment_Middle,
            End       = libphx.TextAlignment_End,
            Justified = libphx.TextAlignment_Justified,

            ToString  = libphx.TextAlignment_ToString,
        }

        if onDef_TextAlignment then onDef_TextAlignment(TextAlignment, mt) end
        TextAlignment = setmetatable(TextAlignment, mt)
    end

    return TextAlignment
end

return Loader
