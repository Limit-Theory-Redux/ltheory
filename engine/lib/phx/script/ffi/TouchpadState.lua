-- TouchpadState ---------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local TouchpadState

do -- C Definitions
    ffi.cdef [[
        typedef struct TouchpadState {} TouchpadState;

        float TouchpadState_GetValue (TouchpadState const*, TouchpadControl control);
    ]]
end

do -- Global Symbol Table
    TouchpadState = {
        GetValue = libphx.TouchpadState_GetValue,
    }

    if onDef_TouchpadState then onDef_TouchpadState(TouchpadState, mt) end
    TouchpadState = setmetatable(TouchpadState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('TouchpadState')
    local mt = {
        __index = {
            getValue = libphx.TouchpadState_GetValue,
        },
    }

    if onDef_TouchpadState_t then onDef_TouchpadState_t(t, mt) end
    TouchpadState_t = ffi.metatype(t, mt)
end

return TouchpadState
