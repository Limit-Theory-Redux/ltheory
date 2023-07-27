-- CursorState -----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local CursorState

do -- C Definitions
    ffi.cdef [[
        typedef struct CursorState {} CursorState;

        float CursorState_GetValue (CursorState const*, CursorControl control);
    ]]
end

do -- Global Symbol Table
    CursorState = {
        GetValue = libphx.CursorState_GetValue,
    }

    if onDef_CursorState then onDef_CursorState(CursorState, mt) end
    CursorState = setmetatable(CursorState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('CursorState')
    local mt = {
        __index = {
            getValue = libphx.CursorState_GetValue,
        },
    }

    if onDef_CursorState_t then onDef_CursorState_t(t, mt) end
    CursorState_t = ffi.metatype(t, mt)
end

return CursorState
