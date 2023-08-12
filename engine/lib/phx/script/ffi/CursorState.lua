-- CursorState -----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local CursorState

do -- C Definitions
    ffi.cdef [[
        float CursorState_Value    (CursorState const*, CursorControl control);
        Vec2f CursorState_Position (CursorState const*);
        bool  CursorState_InWindow (CursorState const*);
    ]]
end

do -- Global Symbol Table
    CursorState = {
        Value    = libphx.CursorState_Value,
        Position = libphx.CursorState_Position,
        InWindow = libphx.CursorState_InWindow,
    }

    if onDef_CursorState then onDef_CursorState(CursorState, mt) end
    CursorState = setmetatable(CursorState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('CursorState')
    local mt = {
        __index = {
            value    = libphx.CursorState_Value,
            position = libphx.CursorState_Position,
            inWindow = libphx.CursorState_InWindow,
        },
    }

    if onDef_CursorState_t then onDef_CursorState_t(t, mt) end
    CursorState_t = ffi.metatype(t, mt)
end

return CursorState
