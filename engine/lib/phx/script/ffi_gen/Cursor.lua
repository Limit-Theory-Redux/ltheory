-- Cursor ----------------------------------------------------------------------

---@class Cursor
---@field Icon fun(self): CursorIcon
---@field SetIcon fun(self, icon: CursorIcon)
---@field IsVisible fun(self): boolean
---@field SetVisible fun(self, visible: boolean)
---@field GrabMode fun(self): CursorGrabMode
---@field SetGrabMode fun(self, grab_mode: CursorGrabMode)
---@field IsHitTest fun(self): boolean
---@field SetHitTest fun(self, hit_test: boolean)

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Cursor {} Cursor;
    ]]

    return 1, 'Cursor'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Cursor

    do -- C Definitions
        ffi.cdef [[
            void           Cursor_Free        (Cursor*);
            CursorIcon     Cursor_Icon        (Cursor const*);
            void           Cursor_SetIcon     (Cursor*, CursorIcon icon);
            bool           Cursor_IsVisible   (Cursor const*);
            void           Cursor_SetVisible  (Cursor*, bool visible);
            CursorGrabMode Cursor_GrabMode    (Cursor const*);
            void           Cursor_SetGrabMode (Cursor*, CursorGrabMode grabMode);
            bool           Cursor_IsHitTest   (Cursor const*);
            void           Cursor_SetHitTest  (Cursor*, bool hitTest);
        ]]
    end

    do -- Global Symbol Table
        Cursor = {}

        if onDef_Cursor then onDef_Cursor(Cursor, mt) end
        Cursor = setmetatable(Cursor, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Cursor')
        local mt = {
            __index = {
                ---@return CursorIcon
                icon        = libphx.Cursor_Icon,
                ---@param icon CursorIcon
                setIcon     = libphx.Cursor_SetIcon,
                ---@return boolean
                isVisible   = libphx.Cursor_IsVisible,
                ---@param visible boolean
                setVisible  = libphx.Cursor_SetVisible,
                ---@return CursorGrabMode
                grabMode    = libphx.Cursor_GrabMode,
                ---@param grab_mode CursorGrabMode
                setGrabMode = libphx.Cursor_SetGrabMode,
                ---@return boolean
                isHitTest   = libphx.Cursor_IsHitTest,
                ---@param hit_test boolean
                setHitTest  = libphx.Cursor_SetHitTest,
            },
        }

        if onDef_Cursor_t then onDef_Cursor_t(t, mt) end
        Cursor_t = ffi.metatype(t, mt)
    end

    return Cursor
end

return Loader
