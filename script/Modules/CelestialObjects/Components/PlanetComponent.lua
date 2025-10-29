local Component = require("Core.ECS.Component")

---@class PlanetComponent: Component
---@field cloudLevel number
---@field oceanLevel number
---@field atmoScale number
---@field color1 Vec3f 
---@field color2 Vec3f 
---@field color3 Vec3f
---@field color4 Vec3f 
---@overload fun(self: PlanetComponent): PlanetComponent subclass internal
---@overload fun(): PlanetComponent subclass external
local PlanetComponent = Subclass("PlanetComponent", Component, function(self)
    self:setComponentName("PlanetComponent")
end)

return PlanetComponent
