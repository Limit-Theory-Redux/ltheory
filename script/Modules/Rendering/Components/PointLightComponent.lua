local Component = require("Core.ECS.Component")

---@class PointLightComponent: Component
---@field color Vec3f Light color (RGB)
---@field intensity number Light intensity multiplier
---@field radius number Light attenuation radius
local PointLightComponent = Subclass("PointLightComponent", Component, function(self, color, intensity, radius)
    self:setComponentName("PointLightComponent")
    self.color = color or Vec3f(1, 1, 1)
    self.intensity = intensity or 1.0
    self.radius = radius or 100.0
end)

function PointLightComponent:getColor()
    return self.color
end

function PointLightComponent:setColor(r, g, b)
    self.color.x = r
    self.color.y = g
    self.color.z = b
end

function PointLightComponent:getIntensity()
    return self.intensity
end

function PointLightComponent:setIntensity(intensity)
    self.intensity = intensity
end

function PointLightComponent:getRadius()
    return self.radius
end

function PointLightComponent:setRadius(radius)
    self.radius = radius
end

--- Get the effective light color (color * intensity)
function PointLightComponent:getEffectiveColor()
    return Vec3f(
        self.color.x * self.intensity,
        self.color.y * self.intensity,
        self.color.z * self.intensity
    )
end

return PointLightComponent
