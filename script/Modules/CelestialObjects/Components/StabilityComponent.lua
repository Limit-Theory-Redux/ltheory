local Component = require("Core.ECS.Component")

---@class StabilityComponent: Component
---@overload fun(self: StabilityComponent, stability: GenStarSystemStability): StabilityComponent subclass internal
---@overload fun(stability: GenStarSystemStability): StabilityComponent subclass external
local StabilityComponent = Subclass("StabilityComponent", Component, function(self, stability)
    self:setComponentName("CelestialStabilityComponent")

    self:setStability(stability)
end)

---@param stability GenStarSystemStability
function StabilityComponent:setStability(stability)
    self.stability = stability
end

---@return GenStarSystemStability|nil stability
function StabilityComponent:getStability()
    return self.stability
end

return StabilityComponent
