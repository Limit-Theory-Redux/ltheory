-- MouseState ------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local MouseState

do -- C Definitions
    ffi.cdef [[
        float MouseState_GetValue (MouseState const*, MouseControl control);
    ]]
end

do -- Global Symbol Table
    MouseState = {
        GetValue = libphx.MouseState_GetValue,
    }

    if onDef_MouseState then onDef_MouseState(MouseState, mt) end
    MouseState = setmetatable(MouseState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('MouseState')
    local mt = {
        __index = {
            getValue = libphx.MouseState_GetValue,
        },
    }

    if onDef_MouseState_t then onDef_MouseState_t(t, mt) end
    MouseState_t = ffi.metatype(t, mt)
end

return MouseState
