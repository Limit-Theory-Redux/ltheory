local Component = require("Core.ECS.Component")

---@class EccentricityComponent: Component
---@overload fun(self: EccentricityComponent, eccentricity: number): EccentricityComponent subclass internal
---@overload fun(eccentricity: number): EccentricityComponent subclass external
local EccentricityComponent = Subclass("EccentricityComponent", Component, function(self, eccentricity)
    self:setComponentName("CelestialEccentricityComponent")
    self:setEccentricity(eccentricity)
end)

---@param eccentricity number
function EccentricityComponent:setEccentricity(eccentricity)
    self.eccentricity = eccentricity
end

---@return number|nil eccentricity
function EccentricityComponent:getEccentricity()
    return self.eccentricity
end

return EccentricityComponent
