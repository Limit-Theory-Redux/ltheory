local Component = require("Core.ECS.Component")

---@class AsteroidComponent: Component
---@overload fun(self: AsteroidComponent): AsteroidComponent subclass internal
---@overload fun(): AsteroidComponent subclass external
--- Marks an entity as a single asteroid. It holds no data: systems find asteroids by viewing it.
local AsteroidComponent = Subclass("AsteroidComponent", Component, function(self)
    self:setComponentName("Asteroid")
end)

return AsteroidComponent
