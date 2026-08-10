local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")

--- Orbital simulation system — updates celestial body positions along their orbits
--- and rotates planets/moons on their axes. Kinematic rigid bodies are moved via setPos().
---@class OrbitalSystem
local OrbitalSystem = {}

--- Collect all orbiting bodies and followers from a star system hierarchy
---@param starSystem Entity The star system root entity
---@return table[] orbiters, table[] followers
function OrbitalSystem:collectOrbiters(starSystem)
    local orbiters = {}
    local followers = {}
    self:_walkForOrbiters(starSystem, nil, orbiters, followers)
    return orbiters, followers
end

--- Get current position of an entity from its rigid body or transform
---@param entity Entity|nil
---@return Position
local function getEntityPos(entity)
    if not entity then return Position(0, 0, 0) end

    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if rbCmp then
        local rb = rbCmp:getRigidBody()
        if rb then return rb:getPos() end
    end

    local transform = entity:get(PhysicsComponents.Transform)
    if transform then return transform:getPos() end

    return Position(0, 0, 0)
end

---@param entity Entity
---@param parentEntity Entity|nil
---@param orbiters table[]
function OrbitalSystem:_walkForOrbiters(entity, parentEntity, orbiters, followers)
    local orbitCmp = entity:get(SpatialComponents.Orbit)
    if orbitCmp then
        local orbitRadius = orbitCmp:getOrbitRadius() or 0
        if orbitRadius > 0 then
            -- Initial phase derived from the entity's CURRENT position,
            -- not randomized: the generator places bodies via
            -- meanAnomaly/true-anomaly math, so a random phase here would
            -- make the first orbital update JUMP the body to a different
            -- point on its orbit (spawning a ship "at the planet" then
            -- leaving it floating at the orbit path as the planet moves).
            -- phase = atan2(z, x) of (entityPos - parentPos) keeps the
            -- orbit continuous with the generated position.
            local entPos = getEntityPos(entity)
            local parentPos = getEntityPos(parentEntity)
            local relX = entPos.x - parentPos.x
            local relZ = entPos.z - parentPos.z
            local phase = math.atan2(relZ, relX)
            if phase ~= phase then phase = 0 end -- NaN guard (relX=relZ=0)

            -- Kepler-ish orbital speed: speed ∝ 1/sqrt(r)
            -- Reduced so travel drive can catch planets
            local speed = 0.5 / math.sqrt(math.max(1, orbitRadius))

            -- Inclination
            local incCmp = entity:get(SpatialComponents.Inclination)
            local inclination = incCmp and math.rad(incCmp:getInclination() or 0) or 0

            table.insert(orbiters, {
                entity       = entity,
                parentEntity = parentEntity,  -- live reference, read position each frame
                orbitRadius  = orbitRadius,
                speed        = speed,
                phase        = phase,
                inclination  = inclination,
            })
        end
    end

    -- If no orbit but has a parent, this entity follows its parent (e.g., ring on planet)
    if not orbitCmp and parentEntity then
        local name = tostring(entity)
        if name:find("AsteroidRingEntity") or name:find("AsteroidBeltEntity") then
            table.insert(followers, {
                entity       = entity,
                parentEntity = parentEntity,
            })
        end
    end

    -- Recurse into children — this entity becomes the parent
    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:_walkForOrbiters(child, entity, orbiters, followers)
        end
    end
end

--- Update all orbiter positions for one frame
---@param orbiters table[] Array from collectOrbiters
---@param dt number Delta time
function OrbitalSystem:update(orbiters, dt, followers)
    for _, orb in ipairs(orbiters) do
        orb.phase = orb.phase + orb.speed * dt

        local x = math.cos(orb.phase) * orb.orbitRadius
        local z = math.sin(orb.phase) * orb.orbitRadius

        -- Apply inclination
        local cosInc = math.cos(orb.inclination)
        local sinInc = math.sin(orb.inclination)
        local y = z * sinInc
        z = z * cosInc

        -- Read parent's CURRENT position (not a stale snapshot)
        local parentPos = getEntityPos(orb.parentEntity)

        local newPos = Position(
            parentPos.x + x,
            parentPos.y + y,
            parentPos.z + z
        )

        -- Update rigid body position (kinematic)
        local rbCmp = orb.entity:get(PhysicsComponents.RigidBody)
        if rbCmp then
            local rb = rbCmp:getRigidBody()
            if rb then
                rb:setPos(newPos)
            end
        end

        -- Update transform component to stay in sync
        local transform = orb.entity:get(PhysicsComponents.Transform)
        if transform then
            transform:setPos(newPos)
        end

        -- Self-rotation (spin on axis)
        local rotPeriod = orb.entity:get(CelestialComponents.RotationPeriod)
        if rotPeriod and rbCmp then
            local period = rotPeriod:getRotationPeriod() or 24
            local rotSpeed = (2 * math.pi) / (period * 2)
            local rb = rbCmp:getRigidBody()
            if rb then
                local currentRot = rb:getRot()
                local deltaRot = Quat.FromAxisAngle(Vec3f(0, 1, 0), rotSpeed * dt)
                rb:setRot(currentRot:mul(deltaRot))
            end
        end
    end

    -- Update followers: entities that track their parent position (rings on planets)
    if followers then
        for _, fol in ipairs(followers) do
            local parentPos = getEntityPos(fol.parentEntity)
            local transform = fol.entity:get(PhysicsComponents.Transform)
            if transform then
                transform:setPos(parentPos)
            end
        end
    end
end

return OrbitalSystem
