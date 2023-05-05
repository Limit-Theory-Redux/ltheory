local libphx = require('ffi.libphx').lib

function onDef_Input (t, mt)
  t.GetMouseDelta    = function () local v = Vec2i() libphx.Input_GetMouseDelta   (v) return v end
  t.GetMousePosition = function () local v = Vec2i() libphx.Input_GetMousePosition(v) return v end
  t.GetMouseScroll   = function () local v = Vec2i() libphx.Input_GetMouseScroll  (v) return v end
  t.SetMousePosition = function (x,y) local v = Vec2i(x,y) libphx.Input_SetMousePosition(v) print(v) return v end
  t.SetMouseScroll   = function () local v = Vec2i() libphx.Input_SetMouseScroll  (v) return v end
end
