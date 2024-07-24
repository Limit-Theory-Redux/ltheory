local libphx = require('libphx').lib

function onDef_WindowImpl_t(t, mt)
    mt.__index.getPosition      = function(self)
        libphx.WindowImpl_Position(self, v)
    end
    mt.__index.getSize          = function(self)
        libphx.WindowImpl_Size(self, v)
    end
    mt.__index.setMousePosition = function(self, x, y)
        local v = Vec2i(x, y)
        libphx.WindowImpl_SetMousePosition(self, v)
        return v
    end
end
