local Component = require("Core.ECS.Component")

---@class WidthComponent: Component
---@overload fun(self: WidthComponent, width: gameunit): WidthComponent subclass internal
---@overload fun(width: gameunit): WidthComponent subclass external
local WidthComponent = Subclass("WidthComponent", Component, function(self, width)
    self:setComponentName("CelestialWidthComponent")
    self:setWidth(width)
end)

---@param width gameunit
function WidthComponent:setWidth(width)
    self.width = width
end

---@return gameunit|nil width
function WidthComponent:getWidth()
    return self.width
end

return WidthComponent
