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
            MouseControl MouseControl_Left;
            MouseControl MouseControl_Middle;
            MouseControl MouseControl_Right;
            MouseControl MouseControl_Forward;
            MouseControl MouseControl_Back;
            MouseControl MouseControl_X1;
            MouseControl MouseControl_X2;
            MouseControl MouseControl_DeltaX;
            MouseControl MouseControl_DeltaY;
            MouseControl MouseControl_ScrollX;
            MouseControl MouseControl_ScrollY;
            MouseControl MouseControl_ScrollPixelX;
            MouseControl MouseControl_ScrollPixelY;

            cstr         MouseControl_ToString(MouseControl);
        ]]
    end

    do -- Global Symbol Table
        MouseControl = {
            Left         = libphx.MouseControl_Left,
            Middle       = libphx.MouseControl_Middle,
            Right        = libphx.MouseControl_Right,
            Forward      = libphx.MouseControl_Forward,
            Back         = libphx.MouseControl_Back,
            X1           = libphx.MouseControl_X1,
            X2           = libphx.MouseControl_X2,
            DeltaX       = libphx.MouseControl_DeltaX,
            DeltaY       = libphx.MouseControl_DeltaY,
            ScrollX      = libphx.MouseControl_ScrollX,
            ScrollY      = libphx.MouseControl_ScrollY,
            ScrollPixelX = libphx.MouseControl_ScrollPixelX,
            ScrollPixelY = libphx.MouseControl_ScrollPixelY,

            ToString     = libphx.MouseControl_ToString,
        }

        if onDef_MouseControl then onDef_MouseControl(MouseControl, mt) end
        MouseControl = setmetatable(MouseControl, mt)
    end

    return MouseControl
end

return Loader
