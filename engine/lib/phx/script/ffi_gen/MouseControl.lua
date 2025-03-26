-- AUTO GENERATED. DO NOT MODIFY!
-- MouseControl ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 MouseControl;
    ]]

    return 2, 'MouseControl'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MouseControl

    do -- C Definitions
        ffi.cdef [[
            cstr         MouseControl_ToString(MouseControl);
        ]]
    end

    do -- Global Symbol Table
        MouseControl = {
            Left         = 0,
            Middle       = 1,
            Right        = 2,
            Forward      = 3,
            Back         = 4,
            X1           = 5,
            X2           = 6,
            DeltaX       = 7,
            DeltaY       = 8,
            ScrollX      = 9,
            ScrollY      = 10,
            ScrollPixelX = 11,
            ScrollPixelY = 12,

            ToString     = libphx.MouseControl_ToString,
        }

        if onDef_MouseControl then onDef_MouseControl(MouseControl, mt) end
        MouseControl = setmetatable(MouseControl, mt)
    end

    return MouseControl
end

return Loader
