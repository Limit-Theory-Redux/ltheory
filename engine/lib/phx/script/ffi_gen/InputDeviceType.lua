-- AUTO GENERATED. DO NOT MODIFY!
-- InputDeviceType -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 InputDeviceType;
    ]]

    return 2, 'InputDeviceType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InputDeviceType

    do -- C Definitions
        ffi.cdef [[
            cstr            InputDeviceType_ToString(InputDeviceType);
        ]]
    end

    do -- Global Symbol Table
        InputDeviceType = {
            Cursor      = 0,
            Gamepad     = 1,
            Keyboard    = 2,
            Mouse       = 3,
            Touchpad    = 4,
            SystemEvent = 5,

            ToString    = libphx.InputDeviceType_ToString,
        }

        if onDef_InputDeviceType then onDef_InputDeviceType(InputDeviceType, mt) end
        InputDeviceType = setmetatable(InputDeviceType, mt)
    end

    return InputDeviceType
end

return Loader
