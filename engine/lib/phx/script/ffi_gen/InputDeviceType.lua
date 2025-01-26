-- AUTO GENERATED. DO NOT MODIFY!
-- InputDeviceType -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 InputDeviceType;
    ]]

    return 2, 'InputDeviceType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InputDeviceType

    do -- C Definitions
        ffi.cdef [[
            InputDeviceType InputDeviceType_Cursor;
            InputDeviceType InputDeviceType_Gamepad;
            InputDeviceType InputDeviceType_Keyboard;
            InputDeviceType InputDeviceType_Mouse;
            InputDeviceType InputDeviceType_Touchpad;
            InputDeviceType InputDeviceType_SystemEvent;

            cstr            InputDeviceType_ToString(InputDeviceType);
        ]]
    end

    do -- Global Symbol Table
        InputDeviceType = {
            Cursor      = libphx.InputDeviceType_Cursor,
            Gamepad     = libphx.InputDeviceType_Gamepad,
            Keyboard    = libphx.InputDeviceType_Keyboard,
            Mouse       = libphx.InputDeviceType_Mouse,
            Touchpad    = libphx.InputDeviceType_Touchpad,
            SystemEvent = libphx.InputDeviceType_SystemEvent,

            ToString    = libphx.InputDeviceType_ToString,
        }

        if onDef_InputDeviceType then onDef_InputDeviceType(InputDeviceType, mt) end
        InputDeviceType = setmetatable(InputDeviceType, mt)
    end

    return InputDeviceType
end

return Loader
