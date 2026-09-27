local libphx = require('libphx').lib

-- ClipRect's C functions all take the current Renderer as their first
-- argument now (see doc/engine/render-thread.md); inject the global
-- `Renderer` set by SetEngine so call sites don't change.
function onDef_ClipRect(t, mt)
    t.Push = function(x, y, sx, sy)
        libphx.ClipRect_Push(Renderer, x, y, sx, sy)
    end

    t.PushCombined = function(x, y, sx, sy)
        libphx.ClipRect_PushCombined(Renderer, x, y, sx, sy)
    end

    t.PushDisabled = function()
        libphx.ClipRect_PushDisabled(Renderer)
    end

    t.PushTransform = function(tx, ty, sx, sy)
        libphx.ClipRect_PushTransform(Renderer, tx, ty, sx, sy)
    end

    t.Pop = function()
        libphx.ClipRect_Pop(Renderer)
    end

    t.PopTransform = function()
        libphx.ClipRect_PopTransform(Renderer)
    end
end
