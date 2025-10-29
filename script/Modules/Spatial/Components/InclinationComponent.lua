local Component = require("Core.ECS.Component")

---@class InclinationComponent: Component
---@overload fun(self: InclinationComponent, inclination: number): InclinationComponent subclass internal
---@overload fun(inclination: number): InclinationComponent subclass external
local InclinationComponent = Subclass("InclinationComponent", Component, function(self, inclination)
    self:setComponentName("CelestialInclinationComponent")
    self:setInclination(inclination)
end)

---@param inclination number
function InclinationComponent:setInclination(inclination)
    self.inclination = inclination
end

---@return number|nil inclination
function InclinationComponent:getInclination()
    return self.inclination
end

return InclinationComponent
