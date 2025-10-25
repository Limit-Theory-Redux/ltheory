local Component = require("Core.ECS.Component")

---@class AtmosphereComponent: Component
---@overload fun(self: AtmosphereComponent): AtmosphereComponent subclass internal
---@overload fun(): AtmosphereComponent subclass external
local AtmosphereComponent = Subclass("AtmosphereComponent", Component, function(self)
    self:setComponentName("CelestialAtmosphereComponent")
end)

return AtmosphereComponent
