local Component = require("Core.ECS.Component")

---@class AsteroidBeltComponent: Component
---@overload fun(self: AsteroidBeltComponent): AsteroidBeltComponent subclass internal
---@overload fun(): AsteroidBeltComponent subclass external
--- Marks an entity as an asteroid belt. It holds no data (the rocks are in `AsteroidBeltData`):
--- systems find belts by viewing it.
local AsteroidBeltComponent = Subclass("AsteroidBeltComponent", Component, function(self)
    self:setComponentName("AsteroidBelt")
end)

return AsteroidBeltComponent
