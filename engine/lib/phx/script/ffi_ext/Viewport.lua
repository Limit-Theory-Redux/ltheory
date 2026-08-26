local libphx = require('libphx').lib

-- Viewport's C functions all take the current Renderer as their first
-- argument now (see ai/multithreaded_rendering.md); inject the global
-- `Renderer` set by SetEngine so call sites (`Viewport.Push(...)` etc.)
-- don't change.
function onDef_Viewport(t, mt)
    t.GetAspect = function()
        return libphx.Viewport_GetAspect(Renderer)
    end

    t.GetSize = function()
        local v = Vec2i()
        libphx.Viewport_GetSize(Renderer, v)
        return v
    end

    t.Push = function(x, y, sx, sy, isWindow)
        libphx.Viewport_Push(Renderer, x, y, sx, sy, isWindow)
    end

    t.Pop = function()
        libphx.Viewport_Pop(Renderer)
    end
end
