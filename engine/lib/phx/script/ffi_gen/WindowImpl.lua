-- WindowImpl ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct WindowImpl {} WindowImpl;
    ]]

    return 1, 'WindowImpl'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local WindowImpl

    do -- C Definitions
        ffi.cdef [[
            void         WindowImpl_Free                      (WindowImpl*);
            void         WindowImpl_BeginDraw                 (WindowImpl const*);
            void         WindowImpl_EndDraw                   (WindowImpl const*);
            cstr         WindowImpl_Title                     (WindowImpl const*);
            void         WindowImpl_SetTitle                  (WindowImpl*, cstr title);
            Cursor*      WindowImpl_Cursor                    (WindowImpl*);
            PresentMode  WindowImpl_PresentMode               (WindowImpl const*);
            void         WindowImpl_SetPresentMode            (WindowImpl*, PresentMode presentMode);
            void         WindowImpl_SetMaximized              (WindowImpl*, bool maximized);
            void         WindowImpl_SetMinimized              (WindowImpl*, bool minimized);
            Vec2i        WindowImpl_Position                  (WindowImpl const*);
            void         WindowImpl_SetCenteredPosition       (WindowImpl*);
            void         WindowImpl_SetPosition               (WindowImpl*, int x, int y);
            float        WindowImpl_Width                     (WindowImpl const*);
            float        WindowImpl_Height                    (WindowImpl const*);
            Vec2f        WindowImpl_Size                      (WindowImpl const*);
            void         WindowImpl_SetSize                   (WindowImpl*, float width, float height);
            uint32       WindowImpl_PhysicalWidth             (WindowImpl const*);
            uint32       WindowImpl_PhysicalHeight            (WindowImpl const*);
            Vec2i        WindowImpl_PhysicalSize              (WindowImpl const*);
            void         WindowImpl_SetPhysicalSize           (WindowImpl*, int width, int height);
            bool         WindowImpl_IsResizable               (WindowImpl const*);
            void         WindowImpl_SetResizable              (WindowImpl*, bool resizable);
            bool         WindowImpl_HasDecorations            (WindowImpl const*);
            void         WindowImpl_SetDecorations            (WindowImpl*, bool decorations);
            bool         WindowImpl_IsFocused                 (WindowImpl const*);
            void         WindowImpl_SetFocused                (WindowImpl*, bool focused);
            void         WindowImpl_SetFullscreen             (WindowImpl*, bool fs, bool exclusive);
            double       WindowImpl_ScaleFactor               (WindowImpl const*);
            Vec2f const* WindowImpl_CursorPosition            (WindowImpl const*);
            void         WindowImpl_SetCursorPosition         (WindowImpl*, Vec2f const* position);
            Vec2f const* WindowImpl_PhysicalCursorPosition    (WindowImpl const*);
            void         WindowImpl_SetPhysicalCursorPosition (WindowImpl*, Vec2d const* position);
        ]]
    end

    do -- Global Symbol Table
        WindowImpl = {}

        if onDef_WindowImpl then onDef_WindowImpl(WindowImpl, mt) end
        WindowImpl = setmetatable(WindowImpl, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('WindowImpl')
        local mt = {
            __index = {
                beginDraw                 = libphx.WindowImpl_BeginDraw,
                endDraw                   = libphx.WindowImpl_EndDraw,
                title                     = libphx.WindowImpl_Title,
                setTitle                  = libphx.WindowImpl_SetTitle,
                cursor                    = libphx.WindowImpl_Cursor,
                presentMode               = libphx.WindowImpl_PresentMode,
                setPresentMode            = libphx.WindowImpl_SetPresentMode,
                setMaximized              = libphx.WindowImpl_SetMaximized,
                setMinimized              = libphx.WindowImpl_SetMinimized,
                position                  = libphx.WindowImpl_Position,
                setCenteredPosition       = libphx.WindowImpl_SetCenteredPosition,
                setPosition               = libphx.WindowImpl_SetPosition,
                width                     = libphx.WindowImpl_Width,
                height                    = libphx.WindowImpl_Height,
                size                      = libphx.WindowImpl_Size,
                setSize                   = libphx.WindowImpl_SetSize,
                physicalWidth             = libphx.WindowImpl_PhysicalWidth,
                physicalHeight            = libphx.WindowImpl_PhysicalHeight,
                physicalSize              = libphx.WindowImpl_PhysicalSize,
                setPhysicalSize           = libphx.WindowImpl_SetPhysicalSize,
                isResizable               = libphx.WindowImpl_IsResizable,
                setResizable              = libphx.WindowImpl_SetResizable,
                hasDecorations            = libphx.WindowImpl_HasDecorations,
                setDecorations            = libphx.WindowImpl_SetDecorations,
                isFocused                 = libphx.WindowImpl_IsFocused,
                setFocused                = libphx.WindowImpl_SetFocused,
                setFullscreen             = libphx.WindowImpl_SetFullscreen,
                scaleFactor               = libphx.WindowImpl_ScaleFactor,
                cursorPosition            = libphx.WindowImpl_CursorPosition,
                setCursorPosition         = libphx.WindowImpl_SetCursorPosition,
                physicalCursorPosition    = libphx.WindowImpl_PhysicalCursorPosition,
                setPhysicalCursorPosition = libphx.WindowImpl_SetPhysicalCursorPosition,
            },
        }

        if onDef_WindowImpl_t then onDef_WindowImpl_t(t, mt) end
        WindowImpl_t = ffi.metatype(t, mt)
    end

    return WindowImpl
end

return Loader
