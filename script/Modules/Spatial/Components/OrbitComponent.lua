local Component = require("Core.ECS.Component")

---@class OrbitComponent: Component
---@overload fun(self: OrbitComponent, orbitRadius: number): OrbitComponent subclass internal
---@overload fun(orbitRadius: number): OrbitComponent subclass external
local OrbitComponent = Subclass("OrbitComponent", Component, function(self, orbitRadius)
    self:setComponentName("CelestialOrbitComponent")
    self:setOrbitRadius(orbitRadius)
end)

---@param orbitRadius number
function OrbitComponent:setOrbitRadius(orbitRadius)
    self.orbitRadius = orbitRadius
end

---@return number|nil orbitRadius
function OrbitComponent:getOrbitRadius()
    return self.orbitRadius
end

return OrbitComponent
