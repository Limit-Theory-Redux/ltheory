-- InputDeviceType -------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local InputDeviceType

do -- C Definitions
    ffi.cdef [[
        typedef uint8 InputDeviceType;

        InputDeviceType InputDeviceType_Cursor;
        InputDeviceType InputDeviceType_Gamepad;
        InputDeviceType InputDeviceType_Keyboard;
        InputDeviceType InputDeviceType_Mouse;
        InputDeviceType InputDeviceType_Touchpad;

        cstr            InputDeviceType_ToString(InputDeviceType);
    ]]
end

do -- Global Symbol Table
    InputDeviceType = {
        Cursor   = libphx.InputDeviceType_Cursor,
        Gamepad  = libphx.InputDeviceType_Gamepad,
        Keyboard = libphx.InputDeviceType_Keyboard,
        Mouse    = libphx.InputDeviceType_Mouse,
        Touchpad = libphx.InputDeviceType_Touchpad,

        ToString = libphx.InputDeviceType_ToString,
    }

    if onDef_InputDeviceType then onDef_InputDeviceType(InputDeviceType, mt) end
    InputDeviceType = setmetatable(InputDeviceType, mt)
end

return InputDeviceType
