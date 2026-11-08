--- Circular-orbit maths on plain numbers: no entities and no globals, so it runs in a worker or a
--- test state. The orbit is in the parent's frame; the phase is measured from the +x axis toward +z
--- and the orbit plane is tilted about the x axis by the inclination. Angles are in radians.
--- Design: doc/script/orbits.md.
local OrbitMath = {}

--- The body's offset from its parent `t` seconds after the phase was taken.
---@param radius number Orbit radius, game units
---@param phase number Radians at t = 0
---@param angularSpeed number Radians per second
---@param inclination number Radians
---@param t number Seconds, may be negative
---@return number x, number y, number z
function OrbitMath.offsetAt(radius, phase, angularSpeed, inclination, t)
    local angle = phase + angularSpeed * t
    local x = math.cos(angle) * radius
    local z = math.sin(angle) * radius
    return x, z * math.sin(inclination), z * math.cos(inclination)
end

--- The body's velocity relative to its parent `t` seconds after the phase was taken.
---@param radius number Orbit radius, game units
---@param phase number Radians at t = 0
---@param angularSpeed number Radians per second
---@param inclination number Radians
---@param t number Seconds, may be negative
---@return number vx, number vy, number vz Game units per second
function OrbitMath.velocityAt(radius, phase, angularSpeed, inclination, t)
    local angle = phase + angularSpeed * t
    local vx = -math.sin(angle) * radius * angularSpeed
    local vz = math.cos(angle) * radius * angularSpeed
    return vx, vz * math.sin(inclination), vz * math.cos(inclination)
end

return OrbitMath
