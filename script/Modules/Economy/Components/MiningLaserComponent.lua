local Component = require("Core.ECS.Component")

---A mining laser on a ship: the ship can take mining jobs (`MiningSystem` runs them).
---@class MiningLaserComponent: Component
---@overload fun(self: MiningLaserComponent): MiningLaserComponent subclass internal
---@overload fun(): MiningLaserComponent subclass external
local MiningLaserComponent = Subclass("MiningLaserComponent", Component, function(self)
    self:setComponentName("EconomyMiningLaser")
end)

return MiningLaserComponent
