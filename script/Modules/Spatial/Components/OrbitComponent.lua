local Component = require("Core.ECS.Component")

---@alias RadiansPerSecond number An angular speed in radians per second

---A circular orbit around the entity's parent, in the parent's frame: its radius, its phase, how fast
---the phase advances and how far the orbit plane is tilted about the x axis. Angles are in radians
---and the phase is measured from the +x axis toward +z.
---@class OrbitComponent: Component
---@overload fun(self: OrbitComponent, orbitRadius: gameunit): OrbitComponent subclass internal
---@overload fun(orbitRadius: gameunit): OrbitComponent subclass external
local OrbitComponent = Subclass("OrbitComponent", Component, function(self, orbitRadius)
    self:setComponentName("CelestialOrbitComponent")
    self:setOrbitRadius(orbitRadius)
    self.phase = 0
    self.angularSpeed = 0
    self.inclination = 0
end)

---@param orbitRadius gameunit
function OrbitComponent:setOrbitRadius(orbitRadius)
    self.orbitRadius = orbitRadius
end

---@return gameunit|nil orbitRadius
function OrbitComponent:getOrbitRadius()
    return self.orbitRadius
end

---@param phase number Radians
function OrbitComponent:setPhase(phase)
    self.phase = phase
end

---@return number phase Radians
function OrbitComponent:getPhase()
    return self.phase
end

---@param angularSpeed RadiansPerSecond
function OrbitComponent:setAngularSpeed(angularSpeed)
    self.angularSpeed = angularSpeed
end

---@return RadiansPerSecond angularSpeed
function OrbitComponent:getAngularSpeed()
    return self.angularSpeed
end

---@param inclination number Radians
function OrbitComponent:setInclination(inclination)
    self.inclination = inclination
end

---@return number inclination Radians
function OrbitComponent:getInclination()
    return self.inclination
end

---Advance the phase by `dt` seconds.
---@param dt number
function OrbitComponent:advance(dt)
    self.phase = self.phase + self.angularSpeed * dt
end

---Where the body is relative to its parent `t` seconds from now, assuming the orbit keeps its speed.
---@param t number Seconds from now, may be negative
---@return gameunit x, gameunit y, gameunit z
function OrbitComponent:offsetAt(t)
    local radius = self.orbitRadius or 0
    local angle = self.phase + self.angularSpeed * t
    local x = math.cos(angle) * radius
    local z = math.sin(angle) * radius
    return x, z * math.sin(self.inclination), z * math.cos(self.inclination)
end

---The body's velocity relative to its parent `t` seconds from now.
---@param t number Seconds from now, may be negative
---@return gameunit vx, gameunit vy, gameunit vz Game units per second
function OrbitComponent:relativeVelocityAt(t)
    local radius = self.orbitRadius or 0
    local angle = self.phase + self.angularSpeed * t
    local vx = -math.sin(angle) * radius * self.angularSpeed
    local vz = math.cos(angle) * radius * self.angularSpeed
    return vx, vz * math.sin(self.inclination), vz * math.cos(self.inclination)
end

return OrbitComponent
