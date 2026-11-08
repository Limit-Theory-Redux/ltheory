local ffi                 = require("ffi")
local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local OrbitalSystem       = require("Modules.CelestialObjects.Systems.OrbitalSystem")

local OPEN_SPACE = PhysicsComponents.GravityWell.OPEN_SPACE

-- A planet's well reaches at least this far past its outermost moon's orbit radius.
local MOON_ORBIT_WELL_FACTOR = 1.5
-- The drive cap starts to fall this many well radii away from a body.
local INFLUENCE_RADIUS_FACTOR = 2.0
-- Largest body move (game units) in one update that is motion; a larger one is a jump (a rebuilt or
-- rebased scene), and the ship's carried velocity is kept through it.
local MAX_FOLLOW_DELTA = 1000

--- Gravity Well System — finds the well each ship with a `GravityWellComponent` is in, carries it with
--- the body it orbits, and gives the travel-drive speed cap at any position.
---@class GravityWellSystem
---@overload fun(): GravityWellSystem
local GravityWellSystem = Class("GravityWellSystem", function(self)
    self.zones = {} -- collected wells: { zoneEntity, parentEntity, parentName, zoneType, radius }
end)

-- The kinds of body that have a well. `nameFromEntity` takes the body's name from its entity name
-- (the text before the id) instead of `bodyName`.
local WELL_BODIES = {
    { marker = CelestialComponents.Star,   zoneType = "star",   bodyName = "Star" },
    { marker = CelestialComponents.Planet, zoneType = "planet", bodyName = "Planet", nameFromEntity = true },
    { marker = CelestialComponents.Moon,   zoneType = "moon",   bodyName = "Moon" },
}

--- The id of an entity's parent, nil without one.
---@param entity Entity
---@return EntityId|nil
local function parentId(entity)
    local parentCmp = entity:get(CoreComponents.Parent)
    local parent = parentCmp and parentCmp:getParent()
    return parent and parent.id
end

--- The radius of the zone each body declares, by body id.
---@return table<EntityId, number>
local function declaredZoneRadii()
    local radii = {}
    for zone in Registry:view(SpatialComponents.Zone) do
        local shapeCmp = zone:get(SpatialComponents.Shape)
        local bodyId = parentId(zone)
        if shapeCmp and bodyId then
            radii[bodyId] = shapeCmp:getRadius()
        end
    end
    return radii
end

--- The largest moon orbit radius around each body, by body id.
---@return table<EntityId, number>
local function outermostMoonOrbits()
    local orbits = {}
    for moon in Registry:view(CelestialComponents.Moon) do
        local orbitCmp = moon:get(SpatialComponents.Orbit)
        local bodyId = parentId(moon)
        if orbitCmp and bodyId then
            orbits[bodyId] = math.max(orbits[bodyId] or 0, orbitCmp:getOrbitRadius() or 0)
        end
    end
    return orbits
end

--- Collect the wells of every star, planet and moon that has a zone (or, for a planet, moons).
function GravityWellSystem:collectZones()
    self.zones = {}
    local declaredRadii = declaredZoneRadii()
    local moonOrbits = outermostMoonOrbits()

    for _, kind in ipairs(WELL_BODIES) do
        for body in Registry:view(kind.marker) do
            local zoneRadius = declaredRadii[body.id]
            local outermostMoonOrbit = moonOrbits[body.id] or 0

            -- For planets: the well must cover the outermost moon orbit
            if kind.zoneType == "planet" and outermostMoonOrbit > 0 then
                local moonBasedRadius = outermostMoonOrbit * MOON_ORBIT_WELL_FACTOR
                zoneRadius = zoneRadius and math.max(zoneRadius, moonBasedRadius) or moonBasedRadius
            end

            if zoneRadius then
                local bodyName = kind.bodyName
                if kind.nameFromEntity then
                    bodyName = tostring(body):match("^(.-)%(") or bodyName
                end
                table.insert(self.zones, {
                    zoneEntity   = body,
                    parentEntity = body,
                    parentName   = bodyName,
                    zoneType     = kind.zoneType,
                    radius       = zoneRadius,
                })
            end
        end
    end
    Log.Info("GravityWellSystem: found %d zones", #self.zones)
end

--- The position of the body a well belongs to, nil if it has no rigid body.
---@param zone table
---@return Position|nil
local function bodyPosition(zone)
    local rbCmp = zone.parentEntity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    return rb and rb:getPos() or nil
end

--- The smallest well that contains `position`, nil in open space.
---@param zones table[]
---@param position Position
---@return table|nil
local function innermostZone(zones, position)
    local best, bestRadius = nil, math.huge
    for _, zone in ipairs(zones) do
        local bodyPos = bodyPosition(zone)
        if bodyPos and position:distance(bodyPos) < zone.radius and zone.radius < bestRadius then
            best, bestRadius = zone, zone.radius
        end
    end
    return best
end

--- The velocity the ship's well body has over the coming physics step: its move from where it was
--- (the previous update's position, or where its rigid body stands when the ship has just entered) to
--- where this step puts it, over `dt`. A jump keeps the ship's carried velocity.
---@param well GravityWellComponent
---@param zone table
---@param dt number
---@return gameunit vx, gameunit vy, gameunit vz Game units per second
local function frameVelocity(well, zone, dt)
    -- The body's position for this step: its rigid body only reaches it when the step runs.
    local now = OrbitalSystem:predictPos(zone.parentEntity, 0)
    local prev = well:getPrevBodyPos() or bodyPosition(zone)
    well:setPrevBodyPos(Position(now.x, now.y, now.z))

    local dx, dy, dz = now.x - prev.x, now.y - prev.y, now.z - prev.z
    if math.max(math.abs(dx), math.abs(dy), math.abs(dz)) > MAX_FOLLOW_DELTA then
        return well:getCarriedVelocity()
    end
    return dx / dt, dy / dt, dz / dt
end

--- Give the ship the frame velocity `(vx, vy, vz)` for the coming step: a force that makes the ship
--- move with the body, as the physics step integrates it. A step of `dt` runs `n` equal substeps of
--- `s = dt/n`. In each one the force adds `g/n` to the solver velocity, where `g = dt·F/m`, and the
--- position moves by `s·w/(1 + s·k)`; the stored velocity is the final solver velocity over
--- `1 + dt·k`. For the carried part `V` of the ship's velocity the step then moves it by
--- `dt·(V + g·(n+1)/(2n))/(1 + s·k)`. The force sets that equal to the body's move `dt·vNew`, and `V`
--- becomes the stored velocity `(V + g)/(1 + dt·k)`. The solver is linear, so the rest of the ship's
--- velocity (thrust, collisions) moves it relative to the body as it would next to a body that
--- stands still; `V` is what the ship's velocity has over that.
---@param well GravityWellComponent
---@param rb RigidBody
---@param dt number
---@param substeps integer The physics step's substep count
---@param vx gameunit
---@param vy gameunit
---@param vz gameunit
local function carry(well, rb, dt, substeps, vx, vy, vz)
    local k = well:getLinearDrag()
    local substepDamping = 1 + dt / substeps * k
    local stepDamping = 1 + dt * k
    local forceScale = 2 * substeps / (substeps + 1) -- from the mean of the substeps' force increments
    local oldX, oldY, oldZ = well:getCarriedVelocity()
    local gx = (vx * substepDamping - oldX) * forceScale
    local gy = (vy * substepDamping - oldY) * forceScale
    local gz = (vz * substepDamping - oldZ) * forceScale
    if gx ~= 0 or gy ~= 0 or gz ~= 0 then
        local scale = rb:getMass() / dt
        -- `applyForce` takes single precision and the engine sums the forces in double: the second call
        -- adds what the first one's rounding left out, so the carry's error doesn't build up as a drift.
        local fx, fy, fz = scale * gx, scale * gy, scale * gz
        local rounded = Vec3f(fx, fy, fz)
        rb:applyForce(rounded)
        rb:applyForce(Vec3f(fx - rounded.x, fy - rounded.y, fz - rounded.z))
    end
    well:setCarriedVelocity((oldX + gx) / stepDamping, (oldY + gy) / stepDamping, (oldZ + gz) / stepDamping)
end

--- Update one ship's well and carry the ship with the body it is in.
---@param shipEntity Entity
---@param well GravityWellComponent
---@param rb RigidBody
---@param dt number
---@param substeps integer
function GravityWellSystem:_updateShip(shipEntity, well, rb, dt, substeps)
    local zone = innermostZone(self.zones, rb:getPos())

    local prevType = well:getZoneType()
    local vx, vy, vz = 0, 0, 0 -- open space carries nothing
    if not zone then
        well:leave()
    else
        if well:getBodyEntity() ~= zone.parentEntity then
            well:enter(zone.parentEntity, zone.parentName, zone.zoneType)
            well:setPrevBodyPos(nil)
        end
        vx, vy, vz = frameVelocity(well, zone, dt)
    end
    carry(well, rb, dt, substeps, vx, vy, vz)

    if well:getZoneType() ~= prevType then
        if zone then
            Log.Info("%s entered %s gravity well (%s)", tostring(shipEntity), zone.parentName, zone.zoneType)
        else
            Log.Info("%s entered open space", tostring(shipEntity))
        end
    end
end

--- Update every ship that has a gravity-well component and a rigid body, before the physics step of
--- `dt` seconds that runs `substeps` solver substeps (`Physics:getSolverSubsteps`). A step of no time
--- (paused) changes nothing.
---@param dt number
---@param substeps integer
function GravityWellSystem:update(dt, substeps)
    assert(substeps >= 1, "a physics step has at least one substep")
    if #self.zones == 0 or dt <= 0 then return end
    -- `Physics:update` takes single precision: the carry has to use the step the engine integrates.
    dt = tonumber(ffi.new("float[1]", dt)[0])
    for shipEntity, well in Registry:iterEntities(PhysicsComponents.GravityWell) do
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        local rb = rbCmp and rbCmp:getRigidBody()
        if rb then
            self:_updateShip(shipEntity, well, rb, dt, substeps)
        end
    end
end

--- Max travel drive speed multiplier at a position — smooth falloff with the distance to each body.
--- Far away = full speed, close to a body = the zone's minimum; the lowest cap wins.
---@param position Position
---@return number maxSpeed
function GravityWellSystem:getMaxDriveSpeedAt(position)
    local cfg = Config.game.travelDrive.maxSpeedByZone
    local maxSpeed = cfg[OPEN_SPACE]

    for _, zone in ipairs(self.zones) do
        local bodyPos = bodyPosition(zone)
        if bodyPos then
            local influenceRadius = zone.radius * INFLUENCE_RADIUS_FACTOR
            local dist = position:distance(bodyPos)
            if dist < influenceRadius then
                -- t: 0 at body center, 1 at edge of influence
                local t = Math.Clamp(dist / influenceRadius, 0, 1)
                local zoneMin = cfg[zone.zoneType] or cfg[OPEN_SPACE]
                local zoneSpeed = zoneMin + (cfg[OPEN_SPACE] - zoneMin) * t * t
                maxSpeed = math.min(maxSpeed, zoneSpeed)
            end
        end
    end

    return maxSpeed
end

return GravityWellSystem()
