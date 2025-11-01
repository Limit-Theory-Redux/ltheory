local Component = require("Core.ECS.Component")

---@class PlanetRingComponent: Component
---@overload fun(self: PlanetRingComponent, genOptions: PlanetRingGenOptions): PlanetRingComponent subclass internal
---@overload fun(genOptions: PlanetRingGenOptions): PlanetRingComponent subclass external
local PlanetRingComponent = Subclass("PlanetRingGenComponent", Component, function(self, genOptions)
    self:setComponentName("CelestialPlanetRingGenComponent")

    ---@class PlanetRingGenOptions
    ---@field innerRadius number
    ---@field outerRadius number
    ---@field rotationSpeed number
    ---@field angle number

    self.innerRadius = genOptions.innerRadius
    self.outerRadius = genOptions.outerRadius
    self.rotationSpeed = genOptions.rotationSpeed
    self.angle = genOptions.angle
end)

return PlanetRingComponent
