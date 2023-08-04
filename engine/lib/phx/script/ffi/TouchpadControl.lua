-- TouchpadControl -------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local TouchpadControl

do -- C Definitions
    ffi.cdef [[
        typedef uint8 TouchpadControl;

        TouchpadControl TouchpadControl_X;
        TouchpadControl TouchpadControl_Y;
        TouchpadControl TouchpadControl_MagnifyDelta;
        TouchpadControl TouchpadControl_RotateDelta;

        cstr            TouchpadControl_ToString(TouchpadControl);
    ]]
end

do -- Global Symbol Table
    TouchpadControl = {
        X            = libphx.TouchpadControl_X,
        Y            = libphx.TouchpadControl_Y,
        MagnifyDelta = libphx.TouchpadControl_MagnifyDelta,
        RotateDelta  = libphx.TouchpadControl_RotateDelta,

        ToString     = libphx.TouchpadControl_ToString,
    }

    if onDef_TouchpadControl then onDef_TouchpadControl(TouchpadControl, mt) end
    TouchpadControl = setmetatable(TouchpadControl, mt)
end

return TouchpadControl
