-- MouseState ------------------------------------------------------------------

---@class MouseState
---@field Value fun(self, control: MouseControl): number
---@field IsPressed fun(self, control: MouseControl): boolean
---@field IsDown fun(self, control: MouseControl): boolean
---@field IsReleased fun(self, control: MouseControl): boolean
---@field Delta fun(self): Vec2
---@field Scroll fun(self): Vec2
---@field ScrollPixel fun(self): Vec2
---@field Position fun(self): Vec2
---@field InWindow fun(self): boolean

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
                ---@return number
                value       = libphx.MouseState_Value,
                ---@param control MouseControl
                ---@return boolean
                isPressed   = libphx.MouseState_IsPressed,
                ---@param control MouseControl
                ---@return boolean
                isDown      = libphx.MouseState_IsDown,
                ---@param control MouseControl
                ---@return boolean
                isReleased  = libphx.MouseState_IsReleased,
                ---@return Vec2
                delta       = libphx.MouseState_Delta,
                ---@return Vec2
                scroll      = libphx.MouseState_Scroll,
                ---@return Vec2
                scrollPixel = libphx.MouseState_ScrollPixel,
                ---@return Vec2
                position    = libphx.MouseState_Position,
                ---@return boolean
                inWindow    = libphx.MouseState_InWindow,
            },
        }

        if onDef_MouseState_t then onDef_MouseState_t(t, mt) end
        MouseState_t = ffi.metatype(t, mt)
    end

    return MouseState
end

return Loader
