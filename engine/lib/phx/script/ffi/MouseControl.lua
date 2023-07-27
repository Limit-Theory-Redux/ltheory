-- MouseControl ----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local MouseControl

do -- C Definitions
    ffi.cdef [[
        typedef uint8 MouseControl;

        MouseControl MouseControl_Left;
        MouseControl MouseControl_Middle;
        MouseControl MouseControl_Right;
        MouseControl MouseControl_X1;
        MouseControl MouseControl_X2;
        MouseControl MouseControl_DeltaX;
        MouseControl MouseControl_DeltaY;
        MouseControl MouseControl_ScrollPixelX;
        MouseControl MouseControl_ScrollPixelY;
        MouseControl MouseControl_ScrollLineX;
        MouseControl MouseControl_ScrollLineY;
        cstr         MouseControl_ToString(MouseControl);
    ]]
end

do -- Global Symbol Table
    MouseControl = {
        Left         = libphx.MouseControl_Left,
        Middle       = libphx.MouseControl_Middle,
        Right        = libphx.MouseControl_Right,
        X1           = libphx.MouseControl_X1,
        X2           = libphx.MouseControl_X2,
        DeltaX       = libphx.MouseControl_DeltaX,
        DeltaY       = libphx.MouseControl_DeltaY,
        ScrollPixelX = libphx.MouseControl_ScrollPixelX,
        ScrollPixelY = libphx.MouseControl_ScrollPixelY,
        ScrollLineX  = libphx.MouseControl_ScrollLineX,
        ScrollLineY  = libphx.MouseControl_ScrollLineY,
        ToString = libphx.MouseControl_ToString,
    }

    if onDef_MouseControl then onDef_MouseControl(MouseControl, mt) end
    MouseControl = setmetatable(MouseControl, mt)
end

return MouseControl
