local Component = require("Core.ECS.Component")

---@class RotationPeriodComponent: Component
---@overload fun(self: RotationPeriodComponent, rotationPeriod: number): RotationPeriodComponent subclass internal
---@overload fun(rotationPeriod: number): RotationPeriodComponent subclass external
local RotationPeriodComponent = Subclass("RotationPeriodComponent", Component, function(self, rotationPeriod)
    self:setComponentName("CelestialRotationPeriodComponent")
    self:setRotationPeriod(rotationPeriod)
end)

---@param rotationPeriod number
function RotationPeriodComponent:setRotationPeriod(rotationPeriod)
    self.rotationPeriod = rotationPeriod
end

---@return number|nil rotationPeriod
function RotationPeriodComponent:getRotationPeriod()
    return self.rotationPeriod
end

return RotationPeriodComponent
