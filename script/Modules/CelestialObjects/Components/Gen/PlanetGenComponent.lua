local Component = require("Core.ECS.Component")

---@class PlanetGenComponent: Component
---@overload fun(self: PlanetGenComponent, genOptions: PlanetGenOptions): PlanetGenComponent subclass internal
---@overload fun(genOptions: PlanetGenOptions): PlanetGenComponent subclass external
local PlanetGenComponent = Subclass("PlanetGenComponent", Component, function(self, genOptions)
    self:setComponentName("CelestialPlanetGenComponent")

    ---@class PlanetGenOptions
    ---@field surfaceFreq number
    ---@field surfacePower number
    ---@field surfaceCoef Vec4f
    ---@field color1 Vec3f
    ---@field color2 Vec3f
    ---@field color3 Vec3f
    ---@field color4 Vec3f
    ---@field oceanLevel number
    ---@field cloudLevel number
    ---@field atmoScale number

    -- Surface
    self.surfaceFreq  = genOptions.surfaceFreq
    self.surfacePower = genOptions.surfacePower
    self.surfaceCoef  = genOptions
    -- Colors
    self.color1       = genOptions.color1
    self.color2       = genOptions.color2
    self.color3       = genOptions.color3
    self.color4       = genOptions.color4
    -- Levels
    self.oceanLevel   = genOptions.oceanLevel
    self.cloudLevel   = genOptions.cloudLevel
    -- Atmosphere
    self.atmoScale    = genOptions.atmoScale
end)

return PlanetGenComponent
