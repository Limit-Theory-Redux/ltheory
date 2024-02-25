-- Cursor ----------------------------------------------------------------------
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
        Cursor = {
            Icon        = libphx.Cursor_Icon,
            SetIcon     = libphx.Cursor_SetIcon,
            IsVisible   = libphx.Cursor_IsVisible,
            SetVisible  = libphx.Cursor_SetVisible,
            GrabMode    = libphx.Cursor_GrabMode,
            SetGrabMode = libphx.Cursor_SetGrabMode,
            IsHitTest   = libphx.Cursor_IsHitTest,
            SetHitTest  = libphx.Cursor_SetHitTest,
        }

        if onDef_Cursor then onDef_Cursor(Cursor, mt) end
        Cursor = setmetatable(Cursor, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Cursor')
        local mt = {
            __index = {
                icon        = libphx.Cursor_Icon,
                setIcon     = libphx.Cursor_SetIcon,
                isVisible   = libphx.Cursor_IsVisible,
                setVisible  = libphx.Cursor_SetVisible,
                grabMode    = libphx.Cursor_GrabMode,
                setGrabMode = libphx.Cursor_SetGrabMode,
                isHitTest   = libphx.Cursor_IsHitTest,
                setHitTest  = libphx.Cursor_SetHitTest,
            },
        }

        if onDef_Cursor_t then onDef_Cursor_t(t, mt) end
        Cursor_t = ffi.metatype(t, mt)
    end

    return Cursor
end

return Loader
