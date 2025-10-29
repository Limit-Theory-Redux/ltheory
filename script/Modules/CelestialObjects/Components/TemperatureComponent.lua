local Component = require("Core.ECS.Component")

---@class TemperatureComponent: Component
---@overload fun(self: TemperatureComponent, temperature: number): TemperatureComponent subclass internal
---@overload fun(temperature: number): TemperatureComponent subclass external
local TemperatureComponent = Subclass("TemperatureComponent", Component, function(self, temperature)
    self:setComponentName("CelestialTemperatureComponent")
    self:setTemperature(temperature)
end)

---@param temperature number
function TemperatureComponent:setTemperature(temperature)
    self.temperature = temperature
end

---@return number|nil temperature
function TemperatureComponent:getTemperature()
    return self.temperature
end

return TemperatureComponent
