local Component = require("Core.ECS.Component")

---@class AsteroidRingComponent: Component
---@overload fun(self: AsteroidRingComponent): AsteroidRingComponent subclass internal
---@overload fun(): AsteroidRingComponent subclass external
--- Marks an entity as a planetary ring of asteroids. It holds no data: systems find rings by
--- viewing it.
local AsteroidRingComponent = Subclass("AsteroidRingComponent", Component, function(self)
    self:setComponentName("AsteroidRing")
end)

return AsteroidRingComponent
