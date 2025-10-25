local Component = require("Core.ECS.Component")

---@class RotationComponent: Component
---@overload fun(self: RotationComponent): RotationComponent subclass internal
---@overload fun(): RotationComponent subclass external
local RotationComponent = Subclass("RotationComponent", Component, function(self)
    self:setComponentName("CelestialRotationComponent")
end)

return RotationComponent
