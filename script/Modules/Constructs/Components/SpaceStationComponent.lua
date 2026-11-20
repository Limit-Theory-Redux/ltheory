local Component = require("Core.ECS.Component")

---@class SpaceStationComponent: Component
---@overload fun(self: SpaceStationComponent): SpaceStationComponent subclass internal
---@overload fun(): SpaceStationComponent subclass external
--- Marks an entity as a space station. It holds no data: systems find stations by viewing it.
local SpaceStationComponent = Subclass("SpaceStationComponent", Component, function(self)
    self:setComponentName("SpaceStation")
end)

return SpaceStationComponent
