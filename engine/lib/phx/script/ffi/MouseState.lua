-- MouseState ------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local MouseState

do -- C Definitions
    ffi.cdef [[
        float MouseState_Value       (MouseState const*, MouseControl control);
        bool  MouseState_IsPressed   (MouseState const*, MouseControl control);
        bool  MouseState_IsDown      (MouseState const*, MouseControl control);
        bool  MouseState_IsReleased  (MouseState const*, MouseControl control);
        Vec2f MouseState_Delta       (MouseState const*);
        Vec2f MouseState_Scroll      (MouseState const*);
        Vec2f MouseState_ScrollPixel (MouseState const*);
        Vec2f MouseState_Position    (MouseState const*);
        bool  MouseState_InWindow    (MouseState const*);
    ]]
end

do -- Global Symbol Table
    MouseState = {
        Value       = libphx.MouseState_Value,
        IsPressed   = libphx.MouseState_IsPressed,
        IsDown      = libphx.MouseState_IsDown,
        IsReleased  = libphx.MouseState_IsReleased,
        Delta       = libphx.MouseState_Delta,
        Scroll      = libphx.MouseState_Scroll,
        ScrollPixel = libphx.MouseState_ScrollPixel,
        Position    = libphx.MouseState_Position,
        InWindow    = libphx.MouseState_InWindow,
    }

    if onDef_MouseState then onDef_MouseState(MouseState, mt) end
    MouseState = setmetatable(MouseState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('MouseState')
    local mt = {
        __index = {
            value       = libphx.MouseState_Value,
            isPressed   = libphx.MouseState_IsPressed,
            isDown      = libphx.MouseState_IsDown,
            isReleased  = libphx.MouseState_IsReleased,
            delta       = libphx.MouseState_Delta,
            scroll      = libphx.MouseState_Scroll,
            scrollPixel = libphx.MouseState_ScrollPixel,
            position    = libphx.MouseState_Position,
            inWindow    = libphx.MouseState_InWindow,
        },
    }

    if onDef_MouseState_t then onDef_MouseState_t(t, mt) end
    MouseState_t = ffi.metatype(t, mt)
end

return MouseState
