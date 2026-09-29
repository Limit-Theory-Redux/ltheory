local Component = require("Core.ECS.Component")

---@class PointLightComponent: Component
---@overload fun(color: Color|Vec3f, radius: number, intensity: number, enabled?: boolean): PointLightComponent
local PointLightComponent = Subclass("PointLightComponent", Component, function(self, color, radius, intensity, enabled)
    self:setComponentName("PointLight")
    self.color = color or Vec3f()
    self.radius = radius or 0
    self.intensity = intensity or 1
    self.enabled = enabled ~= false
    -- Optional bounded samples used by elongated effects such as beams. The
    -- normal single-source case remains driven by the owning entity transform.
    self.sources = nil
end)

---@param color Color|Vec3f
function PointLightComponent:setColor(color)
    self.color = color
end

---@return Color|Vec3f
function PointLightComponent:getColor()
    return self.color
end

---@param radius number
function PointLightComponent:setRadius(radius)
    self.radius = radius
end

---@return number
function PointLightComponent:getRadius()
    return self.radius
end

---@param intensity number
function PointLightComponent:setIntensity(intensity)
    self.intensity = intensity
end

---@return number
function PointLightComponent:getIntensity()
    return self.intensity
end

---@param enabled boolean
function PointLightComponent:setEnabled(enabled)
    self.enabled = enabled == true
end

---@return boolean
function PointLightComponent:isEnabled()
    return self.enabled == true
end

---@param sources table[]|nil
function PointLightComponent:setSources(sources)
    self.sources = sources
end

---@return table[]|nil
function PointLightComponent:getSources()
    return self.sources
end

return PointLightComponent
