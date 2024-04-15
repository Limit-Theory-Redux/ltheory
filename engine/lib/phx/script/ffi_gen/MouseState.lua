-- MouseState ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct MouseState {} MouseState;
    ]]

    return 1, 'MouseState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MouseState

    do -- C Definitions
        ffi.cdef [[
            void  MouseState_Free        (MouseState*);
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
        MouseState = {}

        if onDef_MouseState then onDef_MouseState(MouseState, mt) end
        MouseState = setmetatable(MouseState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('MouseState')
        local mt = {
            __index = {
                ---@param control MouseControl
                ---@return float
                value       = libphx.MouseState_Value,
                ---@param control MouseControl
                ---@return bool
                isPressed   = libphx.MouseState_IsPressed,
                ---@param control MouseControl
                ---@return bool
                isDown      = libphx.MouseState_IsDown,
                ---@param control MouseControl
                ---@return bool
                isReleased  = libphx.MouseState_IsReleased,
                ---@return Vec2f
                delta       = libphx.MouseState_Delta,
                ---@return Vec2f
                scroll      = libphx.MouseState_Scroll,
                ---@return Vec2f
                scrollPixel = libphx.MouseState_ScrollPixel,
                ---@return Vec2f
                position    = libphx.MouseState_Position,
                ---@return bool
                inWindow    = libphx.MouseState_InWindow,
            },
        }

        if onDef_MouseState_t then onDef_MouseState_t(t, mt) end
        MouseState_t = ffi.metatype(t, mt)
    end

    return MouseState
end

return Loader
