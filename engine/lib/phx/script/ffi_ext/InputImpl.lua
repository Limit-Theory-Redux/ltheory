local libphx = require('libphx').lib

function onDef_InputImpl(t, mt)
    t.GetMouseDelta    = function()
        local v = Vec2i()
        libphx.InputImpl_GetMouseDelta(v)
        return v
    end
    t.GetMousePosition = function()
        local v = Vec2i()
        libphx.InputImpl_GetMousePosition(v)
        return v
    end
    t.GetMouseScroll   = function()
        local v = Vec2i()
        libphx.InputImpl_GetMouseScroll(v)
        return v
    end
    t.SetMousePosition = function(x, y)
        local v = Vec2i(x, y)
        libphx.InputImpl_SetMousePosition(v)
        return v
    end
    t.SetMouseScroll   = function()
        local v = Vec2i()
        libphx.InputImpl_SetMouseScroll(v)
        return v
    end
end
