local Component = require("Core.ECS.Component")

---@class DensityComponent: Component
---@overload fun(self: DensityComponent): DensityComponent subclass internal
---@overload fun(): DensityComponent subclass external
local DensityComponent = Subclass("DensityComponent", Component, function(self)
    self:setComponentName("PhysicsDensityComponent")
end)

return DensityComponent
