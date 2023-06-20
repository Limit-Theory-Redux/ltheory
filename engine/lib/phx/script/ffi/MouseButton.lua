-- MouseButton -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local MouseButton

do -- C Definitions
    ffi.cdef [[
    const MouseButton MouseButton_Left;
    const MouseButton MouseButton_Middle;
    const MouseButton MouseButton_Right;
    const MouseButton MouseButton_X1;
    const MouseButton MouseButton_X2;
  ]]
end

do -- Global Symbol Table
    MouseButton = {
        Left = libphx.MouseButton_Left,
        Middle = libphx.MouseButton_Middle,
        Right = libphx.MouseButton_Right,
        X1 = libphx.MouseButton_X1,
        X2 = libphx.MouseButton_X2,
    }

    if onDef_MouseButton then onDef_MouseButton(MouseButton, mt) end
    MouseButton = setmetatable(MouseButton, mt)
end

return MouseButton
