-- Sound -----------------------------------------------------------------------
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
  }

  if onDef_Mouse then onDef_Mouse(Mouse, mt) end
  Mouse = setmetatable(Mouse, mt)
end

return Mouse
