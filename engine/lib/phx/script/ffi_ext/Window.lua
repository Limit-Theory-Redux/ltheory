local libphx = require('libphx').lib

function onDef_Window_t(t, mt)
    ---@class Window
    ---@field public getPosition fun(self: Window): Vec2f
    ---@field public getSize fun(self: Window): Vec2f
    ---@field public setMousePosition fun(self: Window, x: number, y: number): Vec2f
    mt.__index.getPosition      = function(self)
        libphx.Window_Position(self, v)
    end
    mt.__index.getSize          = function(self)
        libphx.Window_Size(self, v)
    end
    mt.__index.setMousePosition = function(self, x, y)
        local v = Vec2f(x, y)
        libphx.Window_SetCursorPosition(self, v)
        return v
    end
end
