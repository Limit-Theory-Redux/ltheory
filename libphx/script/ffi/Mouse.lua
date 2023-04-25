-- Mouse -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Mouse

do -- C Definitions
  ffi.cdef [[
    void   Mouse_GetDelta          (Vec2i* out);
    double Mouse_GetIdleTime       ();
    void   Mouse_GetPosition       (Vec2i* out);
    void   Mouse_GetPositionGlobal (Vec2i* out);
    int    Mouse_GetScroll         ();
    void   Mouse_SetPosition       (int, int);
    void   Mouse_SetVisible        (bool);

    bool   Mouse_Down              (MouseButton);
    bool   Mouse_Pressed           (MouseButton);
    bool   Mouse_Released          (MouseButton);
  ]]
end

do -- Global Symbol Table
  Mouse = {
    GetDelta = libphx.Mouse_GetDelta,
    GetIdleTime = libphx.Mouse_GetIdleTime,
    GetPosition = libphx.Mouse_GetPosition,
    GetPositionGlobal = libphx.Mouse_GetPositionGlobal,
    GetScroll = libphx.Mouse_GetScroll,
    SetPosition = libphx.Mouse_SetPosition,
    SetVisible = libphx.Mouse_SetVisible,
    MouseDown = libphx.Mouse_Down,
    MousePressed = libphx.Mouse_Pressed,
    MouseReleased = libphx.Mouse_Released,
  }

  if onDef_Mouse then onDef_Mouse(Mouse, mt) end
  Mouse = setmetatable(Mouse, mt)
end

return Mouse
