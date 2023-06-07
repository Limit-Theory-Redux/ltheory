local Action = require('GameObjects.Action')

local Orbit = subclass(Action, function(self, target, radius, orbitTime)
    self.target = target
    self.orbitRadius = radius
    self.orbitTime = orbitTime
    self.currentTime = RNG.FromTime():getInt(1, orbitTime)
    self.rAxis = RNG.FromTime():getInt(1, 2)
end)

-- TODO : Compute a closing velocity to get correct lead time
local kLeadTime = 1

function Orbit:clone()
    return Orbit(self.target, self.radius)
end

function Orbit:getName()
    return format('Orbit %s', self.target:getName())
end

function Orbit:onUpdateActive(e, dt)
    if not self.target:isAlive() then
        e:popAction()
        return
    end

    self.currentTime = self.currentTime + dt
    local target = self.target

    local vector = Vec3f()
    -- define 2 axis orbits
    if self.rAxis == 1 then
        vector.x = (math.cos(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.y = 0
        vector.z = (math.sin(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
    elseif self.rAxis == 2 then
        vector.x = (math.cos(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.y = (math.sin(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.z = 0
    end

    self:flyToward(e,
        target:toWorldScaled(vector) + target:getVelocity():scale(kLeadTime),
        target:getForward(),
        target:getUp())
end

return Orbit
