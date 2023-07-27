-- KeyboardState ---------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local KeyboardState

do -- C Definitions
    ffi.cdef [[
        typedef struct KeyboardState {} KeyboardState;

        float KeyboardState_GetValue (KeyboardState const*, KeyboardButton button);
    ]]
end

do -- Global Symbol Table
    KeyboardState = {
        GetValue = libphx.KeyboardState_GetValue,
    }

    if onDef_KeyboardState then onDef_KeyboardState(KeyboardState, mt) end
    KeyboardState = setmetatable(KeyboardState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('KeyboardState')
    local mt = {
        __index = {
            getValue = libphx.KeyboardState_GetValue,
        },
    }

    if onDef_KeyboardState_t then onDef_KeyboardState_t(t, mt) end
    KeyboardState_t = ffi.metatype(t, mt)
end

return KeyboardState
