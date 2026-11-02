local Component = require("Core.ECS.Component")

---@class PlanetComponent: Component
---@overload fun(self: PlanetComponent): PlanetComponent subclass internal
---@overload fun(): PlanetComponent subclass external
--- Marks an entity as a planet. It holds no data: systems find planets by viewing it.
local PlanetComponent = Subclass("PlanetComponent", Component, function(self)
    self:setComponentName("Planet")
end)

return PlanetComponent
