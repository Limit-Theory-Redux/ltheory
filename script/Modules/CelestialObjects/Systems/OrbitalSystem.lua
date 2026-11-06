local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local EntityPosition      = require("Shared.Helpers.EntityPosition")

local getEntityPos = EntityPosition.Get

-- Orbital angular speed is `ORBIT_SPEED_SCALE / sqrt(radius)` (Kepler-like, reduced so that the
-- travel drive can catch planets); radii below `MIN_SPEED_RADIUS` use that radius.
local ORBIT_SPEED_SCALE = 0.5
local MIN_SPEED_RADIUS  = 1
-- Below this the cosine of an inclination is too small to recover a phase from the z offset.
local MIN_INCLINATION_COS = 1e-6
local SECONDS_PER_HOUR    = 3600
local DEFAULT_ROTATION_PERIOD_HOURS = 24

--- Orbital simulation system. The orbit data lives in each body's `OrbitComponent`; this system
--- sets it up, advances it, predicts positions and velocities from it, and moves the bodies along
--- their orbits and spins planets and moons on their axes. Positions come from the orbit data, never
--- from the parent's rigid body, so a child does not trail its parent by a frame.
---@class OrbitalSystem
local OrbitalSystem = {}

--- The orbit of an entity that has one with a positive radius, nil otherwise.
---@param entity Entity
---@return OrbitComponent|nil
local function orbitOf(entity)
    local orbit = entity:get(SpatialComponents.Orbit)
    if orbit and (orbit:getOrbitRadius() or 0) > 0 then return orbit end
    return nil
end

---@param entity Entity
---@return Entity|nil
local function parentOf(entity)
    local parentCmp = entity:get(CoreComponents.Parent)
    return parentCmp and parentCmp:getParent() or nil
end

--- Collect the orbiting bodies of a hierarchy and the rings and belts that follow their parent, and
--- set up each orbit's phase, speed and inclination from the body's current position.
--- The orbiters come from a walk, which lists a parent before its children (a child's initial
--- phase reads its parent's position); the followers come from the ring and belt views.
---@param rootEntity Entity The root of the hierarchy (a universe or a star system)
---@return table[] orbiters, table[] followers
function OrbitalSystem:collectOrbiters(rootEntity)
    local orbiters = {}
    self:_walkForOrbiters(rootEntity, nil, orbiters)
    return orbiters, self:_collectFollowers()
end

--- The rings and belts that have a parent but no orbit of their own.
---@return table[] followers { entity, parentEntity }
function OrbitalSystem:_collectFollowers()
    local followers = {}
    for _, kind in ipairs({ CelestialComponents.AsteroidRing, CelestialComponents.AsteroidBelt }) do
        for entity in Registry:view(kind) do
            local parentCmp = entity:get(CoreComponents.Parent)
            if parentCmp and not entity:get(SpatialComponents.Orbit) then
                table.insert(followers, {
                    entity       = entity,
                    parentEntity = parentCmp:getParent(),
                })
            end
        end
    end
    return followers
end

--- The phase of an orbit that passes through `offset` (a position relative to the parent), so that
--- the body continues from where it was generated instead of jumping.
---@param offset table Position relative to the parent, with x, y, z
---@param inclination number Radians
---@return number phase Radians, 0 for a body at its parent's position
local function phaseOfOffset(offset, inclination)
    local cosInc = math.cos(inclination)
    -- Undo the inclination to get the offset within the orbit plane.
    local planeZ = math.abs(cosInc) > MIN_INCLINATION_COS and offset.z / cosInc
        or offset.y / math.sin(inclination)
    local phase = math.atan2(planeZ, offset.x)
    if phase ~= phase then return 0 end -- NaN guard (offset is zero)
    return phase
end

--- Set an orbit's phase, angular speed and inclination from where its body is now.
---@param entity Entity
---@param parentEntity Entity|nil
---@param orbit OrbitComponent
local function initialiseOrbit(entity, parentEntity, orbit)
    local entPos = getEntityPos(entity)
    local parentPos = getEntityPos(parentEntity)
    local incCmp = entity:get(SpatialComponents.Inclination)
    local inclination = incCmp and math.rad(incCmp:getInclination() or 0) or 0
    -- The generator places bodies from their anomaly, so a random phase here would make the first
    -- update jump the body to another point of its orbit (a ship spawned at a planet would be left
    -- behind as the planet moves away).
    orbit:setInclination(inclination)
    orbit:setPhase(phaseOfOffset({
        x = entPos.x - parentPos.x,
        y = entPos.y - parentPos.y,
        z = entPos.z - parentPos.z,
    }, inclination))
    orbit:setAngularSpeed(ORBIT_SPEED_SCALE / math.sqrt(math.max(MIN_SPEED_RADIUS, orbit:getOrbitRadius())))
end

---@param entity Entity
---@param parentEntity Entity|nil
---@param orbiters table[]
function OrbitalSystem:_walkForOrbiters(entity, parentEntity, orbiters)
    local orbit = orbitOf(entity)
    if orbit then
        initialiseOrbit(entity, parentEntity, orbit)
        table.insert(orbiters, { entity = entity, orbit = orbit })
    end

    -- Recurse into children — this entity becomes the parent
    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:_walkForOrbiters(child, entity, orbiters)
        end
    end
end

--- Where an entity will be `t` seconds from now: its parent's predicted position plus its orbit offset
--- when it orbits, else where it is now (a body without an orbit is taken as not moving).
--- Needs `collectOrbiters` to have set the orbit up.
---@param entity Entity
---@param t number Seconds from now, may be negative
---@return gameunit x, gameunit y, gameunit z
function OrbitalSystem:_predictXYZ(entity, t)
    local orbit = orbitOf(entity)
    if not orbit then
        local pos = getEntityPos(entity)
        return pos.x, pos.y, pos.z
    end
    local px, py, pz = 0, 0, 0
    local parent = parentOf(entity)
    if parent then px, py, pz = self:_predictXYZ(parent, t) end
    local ox, oy, oz = orbit:offsetAt(t)
    return px + ox, py + oy, pz + oz
end

---@param entity Entity
---@param t number Seconds from now, may be negative
---@return Position
function OrbitalSystem:predictPos(entity, t)
    return Position(self:_predictXYZ(entity, t))
end

--- The velocity an entity has `t` seconds from now: its parent's plus its orbital velocity relative
--- to the parent. Zero for a body without an orbit.
---@param entity Entity
---@param t number Seconds from now, may be negative
---@return gameunit vx, gameunit vy, gameunit vz Game units per second
function OrbitalSystem:velocity(entity, t)
    local orbit = orbitOf(entity)
    if not orbit then return 0, 0, 0 end
    local vx, vy, vz = 0, 0, 0
    local parent = parentOf(entity)
    if parent then vx, vy, vz = self:velocity(parent, t) end
    local rx, ry, rz = orbit:relativeVelocityAt(t)
    return vx + rx, vy + ry, vz + rz
end

--- The rotation a body has after spinning on its axis for `dt` seconds. The rotation period is in
--- hours (24 = an Earth-like day); a body without one keeps its rotation.
---@param entity Entity
---@param rb RigidBody
---@param dt number
---@return Quat
local function spunRotation(entity, rb, dt)
    local rotation = rb:getRot()
    local rotPeriod = entity:get(CelestialComponents.RotationPeriod)
    if not rotPeriod then return rotation end

    local periodHours = rotPeriod:getRotationPeriod() or DEFAULT_ROTATION_PERIOD_HOURS
    local rotSpeed = (2 * math.pi) / (periodHours * SECONDS_PER_HOUR)
    return rotation:mul(Quat.FromAxisAngle(Vec3f(0, 1, 0), rotSpeed * dt))
end

--- Move a body to `pos` over the next physics step, spinning it for `spinDt` seconds (none when
--- nil), and set its transform to `pos`. The rigid body is a kinematic target and not a teleport,
--- so physics sees the body's velocity.
---@param entity Entity
---@param pos Position
---@param spinDt number|nil
local function placeBody(entity, pos, spinDt)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    if rb then
        rb:setKinematicTarget(pos, spunRotation(entity, rb, spinDt or 0))
    end

    local transform = entity:get(PhysicsComponents.Transform)
    if transform then transform:setPos(pos) end
end

--- Update all orbiter positions for one frame
---@param orbiters table[] Array from collectOrbiters
---@param dt number Delta time
---@param followers table[]|nil Array from collectOrbiters
function OrbitalSystem:update(orbiters, dt, followers)
    for _, orb in ipairs(orbiters) do
        orb.orbit:advance(dt)
    end

    for _, orb in ipairs(orbiters) do
        placeBody(orb.entity, self:predictPos(orb.entity, 0), dt)
    end

    -- Followers: entities that track their parent position (rings on planets). The rigid body drives
    -- rendering (mWorld comes from the rigid body), so it follows as well as the transform.
    for _, fol in ipairs(followers or {}) do
        placeBody(fol.entity, self:predictPos(fol.parentEntity, 0))
    end
end

return OrbitalSystem
