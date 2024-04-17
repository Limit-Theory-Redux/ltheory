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
                title                     = libphx.Window_Title,
                setTitle                  = libphx.Window_SetTitle,
                cursor                    = libphx.Window_Cursor,
                presentMode               = libphx.Window_PresentMode,
                setPresentMode            = libphx.Window_SetPresentMode,
                setMaximized              = libphx.Window_SetMaximized,
                setMinimized              = libphx.Window_SetMinimized,
                position                  = libphx.Window_Position,
                setCenteredPosition       = libphx.Window_SetCenteredPosition,
                setPosition               = libphx.Window_SetPosition,
                width                     = libphx.Window_Width,
                height                    = libphx.Window_Height,
                size                      = libphx.Window_Size,
                setSize                   = libphx.Window_SetSize,
                physicalWidth             = libphx.Window_PhysicalWidth,
                physicalHeight            = libphx.Window_PhysicalHeight,
                physicalSize              = libphx.Window_PhysicalSize,
                setPhysicalSize           = libphx.Window_SetPhysicalSize,
                isResizable               = libphx.Window_IsResizable,
                setResizable              = libphx.Window_SetResizable,
                hasDecorations            = libphx.Window_HasDecorations,
                setDecorations            = libphx.Window_SetDecorations,
                isTransparent             = libphx.Window_IsTransparent,
                setTransparent            = libphx.Window_SetTransparent,
                isFocused                 = libphx.Window_IsFocused,
                setFocused                = libphx.Window_SetFocused,
                setFullscreen             = libphx.Window_SetFullscreen,
                toggleFullscreen          = libphx.Window_ToggleFullscreen,
                scaleFactor               = libphx.Window_ScaleFactor,
                cursorPosition            = libphx.Window_CursorPosition,
                setCursorPosition         = libphx.Window_SetCursorPosition,
                physicalCursorPosition    = libphx.Window_PhysicalCursorPosition,
                setPhysicalCursorPosition = libphx.Window_SetPhysicalCursorPosition,
            },
        }

        if onDef_Window_t then onDef_Window_t(t, mt) end
        Window_t = ffi.metatype(t, mt)
    end

    return Window
end

return Loader
