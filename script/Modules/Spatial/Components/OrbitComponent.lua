local Component = require("Core.ECS.Component")

---@class OrbitComponent: Component
---@overload fun(self: OrbitComponent): OrbitComponent subclass internal
---@overload fun(): OrbitComponent subclass external
local OrbitComponent = Subclass("OrbitComponent", Component, function(self)
    self:setComponentName("CelestialOrbitComponent")
end)

return OrbitComponent
