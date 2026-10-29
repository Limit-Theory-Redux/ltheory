local Registry          = require("Core.ECS.Registry")
local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local SpatialComponents = require("Modules.Spatial.Components")
local EntityKind        = require("Shared.Helpers.EntityKind")

local OPEN_SPACE = PhysicsComponents.GravityWell.OPEN_SPACE

-- A planet's well reaches at least this far past its outermost moon's orbit radius.
local MOON_ORBIT_WELL_FACTOR = 1.5
-- The drive cap starts to fall this many well radii away from a body.
local INFLUENCE_RADIUS_FACTOR = 2.0
-- Largest body move (game units) that a ship follows in one update; a larger jump is not followed.
local MAX_FOLLOW_DELTA = 1000
-- Body moves (game units) below this are not followed.
local MIN_FOLLOW_DELTA = 0.01

--- Gravity Well System — finds the well each ship with a `GravityWellComponent` is in, makes it drift
--- with the body it orbits, and gives the travel-drive speed cap at any position.
---@class GravityWellSystem
---@overload fun(): GravityWellSystem
local GravityWellSystem = Class("GravityWellSystem", function(self)
    self.zones = {} -- collected wells: { zoneEntity, parentEntity, parentName, zoneType, radius }
end)

--- Collect all wells from the entity hierarchy
---@param rootEntity Entity
function GravityWellSystem:collectZones(rootEntity)
    self.zones = {}
    self:_walkForZones(rootEntity, nil, nil)
    Log.Info("GravityWellSystem: found %d zones", #self.zones)
end

---@param entity Entity
---@param parentEntity Entity|nil
---@param parentName string|nil
function GravityWellSystem:_walkForZones(entity, parentEntity, parentName)
    -- Detect zone type from the entity kind
    local zoneType = nil
    local bodyName = nil
    if EntityKind.Is(entity, "StarEntity") then
        zoneType = "star"; bodyName = "Star"
    elseif EntityKind.Is(entity, "PlanetEntity") then
        zoneType = "planet"; bodyName = tostring(entity):match("^(.-)%(") or "Planet"
    elseif EntityKind.Is(entity, "MoonEntity") then
        zoneType = "moon"; bodyName = "Moon"
    end

    -- Find zone radius: use the declared zone, but for planets expand to cover all moons
    local childrenCmp = entity:get(CoreComponents.Children)
    if not childrenCmp then return end

    local zoneRadius = nil
    local outermostMoonOrbit = 0

    for child in childrenCmp:iterChildren() do
        if EntityKind.Is(child, "ZoneEntity") then
            local shapeCmp = child:get(SpatialComponents.Shape)
            if shapeCmp then
                zoneRadius = shapeCmp:getRadius()
            end
        elseif EntityKind.Is(child, "MoonEntity") then
            -- Track outermost moon orbit
            local orbitCmp = child:get(SpatialComponents.Orbit)
            if orbitCmp then
                local r = orbitCmp:getOrbitRadius() or 0
                if r > outermostMoonOrbit then outermostMoonOrbit = r end
            end
            self:_walkForZones(child, entity, bodyName)
        else
            self:_walkForZones(child, entity, bodyName)
        end
    end

    -- For planets: the well must cover the outermost moon orbit
    if zoneType == "planet" and outermostMoonOrbit > 0 then
        local moonBasedRadius = outermostMoonOrbit * MOON_ORBIT_WELL_FACTOR
        zoneRadius = zoneRadius and math.max(zoneRadius, moonBasedRadius) or moonBasedRadius
    end

    if zoneRadius and zoneType then
        table.insert(self.zones, {
            zoneEntity   = entity,
            parentEntity = entity,
            parentName   = bodyName,
            zoneType     = zoneType,
            radius       = zoneRadius,
        })
    end
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

--- Move the ship by the distance its body moved since the last update. Only when the ship was in
--- the same well then, so that entering a well does not teleport it.
---@param well GravityWellComponent
---@param rb RigidBody
---@param shipPos Position
---@param bodyPos Position
local function followBody(well, rb, shipPos, bodyPos)
    local prev = well:getPrevBodyPos()
    if prev then
        local dx = Math.Clamp(bodyPos.x - prev.x, -MAX_FOLLOW_DELTA, MAX_FOLLOW_DELTA)
        local dy = Math.Clamp(bodyPos.y - prev.y, -MAX_FOLLOW_DELTA, MAX_FOLLOW_DELTA)
        local dz = Math.Clamp(bodyPos.z - prev.z, -MAX_FOLLOW_DELTA, MAX_FOLLOW_DELTA)
        if math.abs(dx) > MIN_FOLLOW_DELTA or math.abs(dy) > MIN_FOLLOW_DELTA
            or math.abs(dz) > MIN_FOLLOW_DELTA then
            rb:setPos(Position(shipPos.x + dx, shipPos.y + dy, shipPos.z + dz))
        end
    end
    well:setPrevBodyPos(Position(bodyPos.x, bodyPos.y, bodyPos.z))
end

--- Update one ship's well and follow the body it is in.
---@param shipEntity Entity
---@param well GravityWellComponent
---@param rb RigidBody
function GravityWellSystem:_updateShip(shipEntity, well, rb)
    local shipPos = rb:getPos()
    local zone = innermostZone(self.zones, shipPos)

    local prevType = well:getZoneType()
    if not zone then
        well:leave()
    else
        if well:getBodyEntity() ~= zone.parentEntity then
            well:enter(zone.parentEntity, zone.parentName, zone.zoneType)
            well:setPrevBodyPos(nil)
        end
        followBody(well, rb, shipPos, bodyPosition(zone))
    end

    if well:getZoneType() ~= prevType then
        if zone then
            Log.Info("%s entered %s gravity well (%s)", tostring(shipEntity), zone.parentName, zone.zoneType)
        else
            Log.Info("%s entered open space", tostring(shipEntity))
        end
    end
end

--- Update every ship that has a gravity-well component and a rigid body.
function GravityWellSystem:update()
    if #self.zones == 0 then return end
    for shipEntity, well in Registry:iterEntities(PhysicsComponents.GravityWell) do
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        local rb = rbCmp and rbCmp:getRigidBody()
        if rb then
            self:_updateShip(shipEntity, well, rb)
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
