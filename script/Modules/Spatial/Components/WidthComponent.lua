local Component = require("Core.ECS.Component")

---@class WidthComponent: Component
---@overload fun(self: WidthComponent): WidthComponent subclass internal
---@overload fun(): WidthComponent subclass external
local WidthComponent = Subclass("WidthComponent", Component, function(self)
    self:setComponentName("CelestialWidthComponent")
end)

return WidthComponent
