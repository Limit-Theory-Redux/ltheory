local Component = require("Core.ECS.Component")

---@class WidthComponent: Component
---@overload fun(self: WidthComponent, width: number): WidthComponent subclass internal
---@overload fun(width: number): WidthComponent subclass external
local WidthComponent = Subclass("WidthComponent", Component, function(self, width)
    self:setComponentName("CelestialWidthComponent")
    self:setWidth(width)
end)

---@param width number
function WidthComponent:setWidth(width)
    self.width = width
end

---@return number|nil width
function WidthComponent:getWidth()
    return self.width
end

return WidthComponent
