local Component = require("Core.ECS.Component")

---@class EccentricityComponent: Component
---@overload fun(self: EccentricityComponent): EccentricityComponent subclass internal
---@overload fun(): EccentricityComponent subclass external
local EccentricityComponent = Subclass("EccentricityComponent", Component, function(self)
    self:setComponentName("CelestialEccentricityComponent")
end)

return EccentricityComponent
