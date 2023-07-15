-- Window ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Window

do -- C Definitions
    ffi.cdef [[
        void    Window_Free             (Window*);
        Window* Window_Create           (cstr title, WindowPos x, WindowPos y, int sx, int sy, WindowMode mode);
        void    Window_BeginDraw        (Window const*);
        void    Window_EndDraw          (Window const*);
        void    Window_GetPosition      (Window const*, Vec2i* out);
        void    Window_GetSize          (Window const*, Vec2i* out);
        cstr    Window_GetTitle         (Window const*);
        void    Window_SetFullscreen    (Window const*, bool fs);
        void    Window_SetPosition      (Window const*, WindowPos x, WindowPos y);
        void    Window_SetSize          (Window const*, int sx, int sy);
        void    Window_SetTitle         (Window const*, cstr title);
        void    Window_SetVsync         (Window const*, bool vsync);
        void    Window_SetCursor        (Window*, cstr name, int hotx, int hoty);
        void    Window_SetMousePosition (Window const*, Vec2i const* position);
        void    Window_SetWindowGrab    (Window const*, bool grabbed);
        void    Window_ToggleFullscreen (Window*);
        void    Window_Hide             (Window const*);
        void    Window_Show             (Window const*);
    ]]
end

do -- Global Symbol Table
    Window = {
        Free             = libphx.Window_Free,
        Create           = libphx.Window_Create,
        BeginDraw        = libphx.Window_BeginDraw,
        EndDraw          = libphx.Window_EndDraw,
        GetPosition      = libphx.Window_GetPosition,
        GetSize          = libphx.Window_GetSize,
        GetTitle         = libphx.Window_GetTitle,
        SetFullscreen    = libphx.Window_SetFullscreen,
        SetPosition      = libphx.Window_SetPosition,
        SetSize          = libphx.Window_SetSize,
        SetTitle         = libphx.Window_SetTitle,
        SetVsync         = libphx.Window_SetVsync,
        SetCursor        = libphx.Window_SetCursor,
        SetMousePosition = libphx.Window_SetMousePosition,
        SetWindowGrab    = libphx.Window_SetWindowGrab,
        ToggleFullscreen = libphx.Window_ToggleFullscreen,
        Hide             = libphx.Window_Hide,
        Show             = libphx.Window_Show,
    }

    if onDef_Window then onDef_Window(Window, mt) end
    Window = setmetatable(Window, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Window')
    local mt = {
        __index = {
            managed          = function (self) return ffi.gc(self, libphx.Window_Free) end,
            free             = libphx.Window_Free,
            beginDraw        = libphx.Window_BeginDraw,
            endDraw          = libphx.Window_EndDraw,
            getPosition      = libphx.Window_GetPosition,
            getSize          = libphx.Window_GetSize,
            getTitle         = libphx.Window_GetTitle,
            setFullscreen    = libphx.Window_SetFullscreen,
            setPosition      = libphx.Window_SetPosition,
            setSize          = libphx.Window_SetSize,
            setTitle         = libphx.Window_SetTitle,
            setVsync         = libphx.Window_SetVsync,
            setCursor        = libphx.Window_SetCursor,
            setMousePosition = libphx.Window_SetMousePosition,
            setWindowGrab    = libphx.Window_SetWindowGrab,
            toggleFullscreen = libphx.Window_ToggleFullscreen,
            hide             = libphx.Window_Hide,
            show             = libphx.Window_Show,
        },
    }

    if onDef_Window_t then onDef_Window_t(t, mt) end
    Window_t = ffi.metatype(t, mt)
end

return Window
