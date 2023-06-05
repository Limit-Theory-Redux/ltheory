-- Window ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Window

do -- C Definitions
  ffi.cdef [[
    Window* Window_Create           (cstr title, WindowPos x, WindowPos y, int sx, int sy, WindowMode mode);
    void    Window_BeginDraw        (Window*);
    void    Window_EndDraw          (Window*);
    void    Window_GetPosition      (Window*, Vec2i* out);
    void    Window_GetSize          (Window*, Vec2i* out);
    cstr    Window_GetTitle         (Window*);
    void    Window_SetFullscreen    (Window*, bool fs);
    void    Window_SetPosition      (Window*, WindowPos x, WindowPos y);
    void    Window_SetSize          (Window*, int sx, int sy);
    void    Window_SetTitle         (Window*, cstr title);
    void    Window_SetVsync         (Window*, bool vsync);
    void    Window_SetCursor        (Window*, cstr name, int hotx, int hoty);
    void    Window_SetMousePosition (Window*, Vec2i* position);
    void    Window_SetWindowGrab    (Window*, bool grabbed);
    void    Window_ToggleFullscreen (Window*);
    void    Window_Hide             (Window*);
    void    Window_Show             (Window*);
  ]]
end

do -- Global Symbol Table
  Window = {
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

return Window
