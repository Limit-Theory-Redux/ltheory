local Component = require("Core.ECS.Component")

---@class GravityComponent: Component
---@overload fun(self: GravityComponent): GravityComponent subclass internal
---@overload fun(): GravityComponent subclass external
local GravityComponent = Subclass("GravityComponent", Component, function(self)
    self:setComponentName("CelestialGravityComponent")
end)

return GravityComponent
