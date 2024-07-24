local libphx = require('libphx').lib

function onDef_Window_t(t, mt)
    mt.__index.getPosition      = function(self)
        libphx.Window_Position(self, v)
    end
    mt.__index.getSize          = function(self)
        libphx.Window_Size(self, v)
    end
    mt.__index.setMousePosition = function(self, x, y)
        local v = Vec2i(x, y)
        libphx.Window_SetMousePosition(self, v)
        return v
    end
end
