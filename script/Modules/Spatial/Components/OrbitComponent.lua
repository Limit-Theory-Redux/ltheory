local Component = require("Core.ECS.Component")

---@class OrbitComponent: Component
---@overload fun(self: OrbitComponent, orbitRadius: gameunit): OrbitComponent subclass internal
---@overload fun(orbitRadius: gameunit): OrbitComponent subclass external
local OrbitComponent = Subclass("OrbitComponent", Component, function(self, orbitRadius)
    self:setComponentName("CelestialOrbitComponent")
    self:setOrbitRadius(orbitRadius)
end)

---@param orbitRadius gameunit
function OrbitComponent:setOrbitRadius(orbitRadius)
    self.orbitRadius = orbitRadius
end

---@return gameunit|nil orbitRadius
function OrbitComponent:getOrbitRadius()
    return self.orbitRadius
end

return OrbitComponent
