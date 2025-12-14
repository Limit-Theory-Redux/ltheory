local Component = require("Core.ECS.Component")

---@class LuminosityComponent: Component
---@overload fun(self: LuminosityComponent, luminosity: number): LuminosityComponent subclass internal
---@overload fun(luminosity: number): LuminosityComponent subclass external
local LuminosityComponent = Subclass("LuminosityComponent", Component, function(self, luminosity)
    self:setComponentName("CelestialLuminosityComponent")
    self:setLuminosity(luminosity)
end)

---@param Luminosity number
function LuminosityComponent:setLuminosity(Luminosity)
    self.luminosity = Luminosity
end

---@return number|nil Luminosity
function LuminosityComponent:getLuminosity()
    return self.luminosity
end

return LuminosityComponent
