local Component = require("Core.ECS.Component")

---@class ThicknessComponent: Component
---@overload fun(self: ThicknessComponent): ThicknessComponent subclass internal
---@overload fun(): ThicknessComponent subclass external
local ThicknessComponent = Subclass("ThicknessComponent", Component, function(self)
    self:setComponentName("CelestialThicknessComponent")
end)

return ThicknessComponent
