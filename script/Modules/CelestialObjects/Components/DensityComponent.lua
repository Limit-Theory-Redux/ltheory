local Component = require("Core.ECS.Component")

---@class DensityComponent: Component
---@overload fun(self: DensityComponent, density: number): DensityComponent subclass internal
---@overload fun(density: number): DensityComponent subclass external
local DensityComponent = Subclass("DensityComponent", Component, function(self, density)
    self:setComponentName("CelestialDensityComponent")
    self:setDensity(density)
end)

---@param density number
function DensityComponent:setDensity(density)
    self.density = density
end

---@return number|nil density
function DensityComponent:getDensity()
    return self.density
end

return DensityComponent
