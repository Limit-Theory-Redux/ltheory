-- Window ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Window {} Window;
    ]]

    return 1, 'Window'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Window

    do -- C Definitions
        ffi.cdef [[
            void         Window_Free                      (Window*);
            void         Window_BeginDraw                 (Window const*);
            void         Window_EndDraw                   (Window const*);
            cstr         Window_Title                     (Window const*);
            void         Window_SetTitle                  (Window*, cstr title);
            Cursor*      Window_Cursor                    (Window*);
            PresentMode  Window_PresentMode               (Window const*);
            void         Window_SetPresentMode            (Window*, PresentMode presentMode);
            void         Window_SetMaximized              (Window*, bool maximized);
            void         Window_SetMinimized              (Window*, bool minimized);
            Vec2i        Window_Position                  (Window const*);
            void         Window_SetCenteredPosition       (Window*);
            void         Window_SetPosition               (Window*, int x, int y);
            float        Window_Width                     (Window const*);
            float        Window_Height                    (Window const*);
            Vec2f        Window_Size                      (Window const*);
            void         Window_SetSize                   (Window*, float width, float height);
            uint32       Window_PhysicalWidth             (Window const*);
            uint32       Window_PhysicalHeight            (Window const*);
            Vec2i        Window_PhysicalSize              (Window const*);
            void         Window_SetPhysicalSize           (Window*, int width, int height);
            bool         Window_IsResizable               (Window const*);
            void         Window_SetResizable              (Window*, bool resizable);
            bool         Window_HasDecorations            (Window const*);
            void         Window_SetDecorations            (Window*, bool decorations);
            bool         Window_IsTransparent             (Window const*);
            void         Window_SetTransparent            (Window*, bool transparent);
            bool         Window_IsFocused                 (Window const*);
            void         Window_SetFocused                (Window*, bool focused);
            void         Window_SetFullscreen             (Window*, bool fs);
            void         Window_ToggleFullscreen          (Window*);
            double       Window_ScaleFactor               (Window const*);
            Vec2f const* Window_CursorPosition            (Window const*);
            void         Window_SetCursorPosition         (Window*, Vec2f const* position);
            Vec2f const* Window_PhysicalCursorPosition    (Window const*);
            void         Window_SetPhysicalCursorPosition (Window*, Vec2d const* position);
        ]]
    end

    do -- Global Symbol Table
        Window = {}

        if onDef_Window then onDef_Window(Window, mt) end
        Window = setmetatable(Window, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Window')
        local mt = {
            __index = {
                beginDraw                 = libphx.Window_BeginDraw,
                endDraw                   = libphx.Window_EndDraw,
                -- The window title.
                ---@return string
                title                     = libphx.Window_Title,
                -- Set the window title.
                ---@param title string
                setTitle                  = libphx.Window_SetTitle,
                -- The window cursor.
                ---@return Cursor
                cursor                    = libphx.Window_Cursor,
                -- The window present mode.
                ---@return PresentMode
                presentMode               = libphx.Window_PresentMode,
                -- Set window present mode.
                ---@param present_mode PresentMode
                setPresentMode            = libphx.Window_SetPresentMode,
                -- Setting this to true will attempt to maximize the window.
                --
                -- Setting it to false will attempt to un-maximize the window.
                ---@param maximized boolean
                setMaximized              = libphx.Window_SetMaximized,
                -- Setting this to true will attempt to minimize the window.
                --
                -- Setting it to false will attempt to un-minimize the window.
                ---@param minimized boolean
                setMinimized              = libphx.Window_SetMinimized,
                -- The window's client position in physical pixels.
                --
                -- See [`WindowPosition`] for an explanation about logical/physical sizes.
                ---@return IVec2
                position                  = libphx.Window_Position,
                -- Set the window's client position in the center of the current monitor.
                setCenteredPosition       = libphx.Window_SetCenteredPosition,
                -- Set the window's client position in physical pixels.
                --
                -- See [`WindowPosition`] for an explanation about logical/physical sizes.
                ---@param x integer
                ---@param y integer
                setPosition               = libphx.Window_SetPosition,
                -- The window's client area width in logical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return number
                width                     = libphx.Window_Width,
                -- The window's client area height in logical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return number
                height                    = libphx.Window_Height,
                -- The window's client area size in logical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return Vec2
                size                      = libphx.Window_Size,
                -- Set the window's client area size in logical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@param width number
                ---@param height number
                setSize                   = libphx.Window_SetSize,
                -- The window's client area width in physical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return integer
                physicalWidth             = libphx.Window_PhysicalWidth,
                -- The window's client area height in physical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return integer
                physicalHeight            = libphx.Window_PhysicalHeight,
                -- The window's client area size in physical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return IVec2
                physicalSize              = libphx.Window_PhysicalSize,
                -- Set the window's client area size in physical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@param width integer
                ---@param height integer
                setPhysicalSize           = libphx.Window_SetPhysicalSize,
                -- Is the window resizable?
                ---@return boolean
                isResizable               = libphx.Window_IsResizable,
                -- Should the window be resizable?
                ---@param resizable boolean
                setResizable              = libphx.Window_SetResizable,
                -- Has the window decorations?
                ---@return boolean
                hasDecorations            = libphx.Window_HasDecorations,
                -- Should the window have decorations?
                ---@param decorations boolean
                setDecorations            = libphx.Window_SetDecorations,
                -- Is the window transparent?
                ---@return boolean
                isTransparent             = libphx.Window_IsTransparent,
                -- Should the window be transparent?
                ---@param transparent boolean
                setTransparent            = libphx.Window_SetTransparent,
                -- Is the window focused?
                ---@return boolean
                isFocused                 = libphx.Window_IsFocused,
                -- Should the window be focused?
                ---@param focused boolean
                setFocused                = libphx.Window_SetFocused,
                ---@param fs boolean
                setFullscreen             = libphx.Window_SetFullscreen,
                toggleFullscreen          = libphx.Window_ToggleFullscreen,
                -- The window's scale factor.
                --
                -- Ratio of physical size to logical size, see [`WindowResolution`].
                ---@return number
                scaleFactor               = libphx.Window_ScaleFactor,
                -- The cursor position in this window in logical pixels.
                --
                -- Returns `None` if the cursor is outside the window area.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return Vec2
                cursorPosition            = libphx.Window_CursorPosition,
                -- Set the cursor position in this window in logical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@param position Vec2
                setCursorPosition         = libphx.Window_SetCursorPosition,
                -- The cursor position in this window in physical pixels.
                --
                -- Returns `None` if the cursor is outside the window area.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@return Vec2
                physicalCursorPosition    = libphx.Window_PhysicalCursorPosition,
                -- Set the cursor position in this window in physical pixels.
                --
                -- See [`WindowResolution`] for an explanation about logical/physical sizes.
                ---@param position DVec2
                setPhysicalCursorPosition = libphx.Window_SetPhysicalCursorPosition,
            },
        }

        if onDef_Window_t then onDef_Window_t(t, mt) end
        Window_t = ffi.metatype(t, mt)
    end

    return Window
end

return Loader
