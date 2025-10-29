local Component = require("Core.ECS.Component")

---@class TemperatureComponent: Component
---@overload fun(self: TemperatureComponent): AgeComponent subclass internal
---@overload fun(): TemperatureComponent subclass external
local TemperatureComponent = Subclass("TemperatureComponent", Component, function(self)
    self:setComponentName("CelestialTemperatureComponent")
end)

return TemperatureComponent
