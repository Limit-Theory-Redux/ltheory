local Component = require("Core.ECS.Component")

---@class MoonGenComponent: Component
---@overload fun(self: MoonGenComponent, genOptions: MoonGenOptions): MoonGenComponent subclass internal
---@overload fun(genOptions: PlanetGenOptions): MoonGenComponent subclass external
local MoonGenComponent = Subclass("MoonGenComponent", Component, function(self, genOptions)
    self:setComponentName("CelestialMoonGenComponent")

    ---@class MoonGenOptions
    ---@field craterDensity number
    ---@field craterSharpness number
    ---@field mariaAmount number
    ---@field highlandColor Vec3f
    ---@field mariaColor Vec3f
    ---@field heightMult number
    ---@field enableAtmosphere boolean

    -- Surface
    self.craterDensity    = genOptions.craterDensity
    self.craterSharpness  = genOptions.craterSharpness
    self.mariaAmount      = genOptions.mariaAmount
    -- Colors
    self.highlandColor    = genOptions.highlandColor
    self.mariaColor       = genOptions.mariaColor

    -- Levels
    self.heightMult       = genOptions.heightMult
    -- Atmosphere
    self.enableAtmosphere = genOptions.enableAtmosphere
end)

return MoonGenComponent
