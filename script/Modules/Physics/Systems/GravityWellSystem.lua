local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local SpatialComponents = require("Modules.Spatial.Components")

--- Gravity Well System — detects which zone the ship is in,
--- applies velocity matching so the ship drifts with orbiting bodies,
--- and reports zone type for travel drive speed limiting.
---@class GravityWellSystem
---@overload fun(): GravityWellSystem
local GravityWellSystem = Class("GravityWellSystem", function(self)
    self.zones = {}           -- collected zones: { entity, parentEntity, parentName, zoneType, radius }
    self.currentZone = nil    -- the zone the ship is currently in
    self.currentZoneType = "openSpace"
    self.prevBodyPositions = {} -- for velocity matching
end)

--- Collect all zones from the entity hierarchy
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
    local name = tostring(entity)

    -- Detect zone type from entity name
    local zoneType = nil
    local bodyName = nil
    if name:find("StarEntity") then
        zoneType = "star"; bodyName = "Star"
    elseif name:find("PlanetEntity") then
        zoneType = "planet"; bodyName = name:match("^(.-)%(") or "Planet"
    elseif name:find("MoonEntity") then
        zoneType = "moon"; bodyName = "Moon"
    end

    -- Find zone radius: use the declared zone, but for planets expand to cover all moons
    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        local zoneRadius = nil
        local outermostMoonOrbit = 0

        for child in childrenCmp:iterChildren() do
            local childName = tostring(child)
            if childName:find("ZoneEntity") then
                local shapeCmp = child:get(SpatialComponents.Shape)
                if shapeCmp then
                    zoneRadius = shapeCmp:getRadius()
                end
            elseif childName:find("MoonEntity") then
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

        -- For planets: zone must be at least outermost moon orbit * 1.5
        if zoneType == "planet" and outermostMoonOrbit > 0 then
            local moonBasedRadius = outermostMoonOrbit * 1.5
            if zoneRadius then
                zoneRadius = math.max(zoneRadius, moonBasedRadius)
            else
                zoneRadius = moonBasedRadius
            end
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

    -- Also recurse if this entity has no zone children but has children
    if not childrenCmp then return end
end

--- Update: detect which zone the ship is in, apply velocity matching
---@param dt number
---@param shipEntity Entity
function GravityWellSystem:update(dt, shipEntity)
    if not shipEntity or #self.zones == 0 then return end

    local shipRbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not shipRbCmp then return end
    local shipRb = shipRbCmp:getRigidBody()
    if not shipRb then return end
    local shipPos = shipRb:getPos()
    self.shipPos = shipPos -- cache for getMaxDriveSpeed

    -- Find innermost zone the ship is inside
    local bestZone = nil
    local bestRadius = math.huge

    for _, zone in ipairs(self.zones) do
        local parentRbCmp = zone.parentEntity:get(PhysicsComponents.RigidBody)
        if parentRbCmp then
            local parentRb = parentRbCmp:getRigidBody()
            if parentRb then
                local parentPos = parentRb:getPos()
                local dist = shipPos:distance(parentPos)

                if dist < zone.radius and zone.radius < bestRadius then
                    bestZone = zone
                    bestRadius = zone.radius
                end
            end
        end
    end

    -- Update current zone
    local prevZoneType = self.currentZoneType
    if bestZone then
        self.currentZone = bestZone
        self.currentZoneType = bestZone.zoneType
    else
        self.currentZone = nil
        self.currentZoneType = "openSpace"
    end

    -- Log zone transitions
    if self.currentZoneType ~= prevZoneType then
        if self.currentZone then
            Log.Info("Entered %s gravity well (%s)", self.currentZone.parentName, self.currentZoneType)
        else
            Log.Info("Entered open space")
        end
    end

    -- Velocity matching: move ship with the body it's orbiting near
    -- Only apply delta if we were in the SAME zone last frame (prevents teleporting on zone entry)
    if bestZone and self.prevZoneBody == bestZone.parentEntity then
        local key = bestZone.parentEntity
        local parentRb = bestZone.parentEntity:get(PhysicsComponents.RigidBody):getRigidBody()
        local currentPos = parentRb:getPos()

        local prevPos = self.prevBodyPositions[key]
        if prevPos then
            local dx = currentPos.x - prevPos.x
            local dy = currentPos.y - prevPos.y
            local dz = currentPos.z - prevPos.z

            -- Clamp delta to prevent teleporting (max ~1000 units per frame)
            local maxDelta = 1000
            dx = Math.Clamp(dx, -maxDelta, maxDelta)
            dy = Math.Clamp(dy, -maxDelta, maxDelta)
            dz = Math.Clamp(dz, -maxDelta, maxDelta)

            if math.abs(dx) > 0.01 or math.abs(dy) > 0.01 or math.abs(dz) > 0.01 then
                shipRb:setPos(Position(shipPos.x + dx, shipPos.y + dy, shipPos.z + dz))
            end
        end
    end

    -- Update previous body position for next frame's delta
    if bestZone then
        local parentRb = bestZone.parentEntity:get(PhysicsComponents.RigidBody):getRigidBody()
        self.prevBodyPositions[bestZone.parentEntity] = Position(
            parentRb:getPos().x, parentRb:getPos().y, parentRb:getPos().z)
    end
    self.prevZoneBody = bestZone and bestZone.parentEntity or nil
end

--- Get current zone type ("openSpace", "star", "planet", "moon")
---@return string
function GravityWellSystem:getZoneType()
    return self.currentZoneType
end

--- Get current zone name for HUD display
---@return string
function GravityWellSystem:getZoneName()
    if self.currentZone then
        return self.currentZone.parentName .. " (" .. self.currentZoneType .. ")"
    end
    return "Open Space"
end

--- Get max travel drive speed — smooth linear falloff based on distance to nearest body
--- Far away = full speed, close to body = zone minimum speed
---@return number maxSpeed, string zoneName
function GravityWellSystem:getMaxDriveSpeed()
    local cfg = Config.game.travelDrive.maxSpeedByZone
    local maxSpeed = cfg.openSpace

    if not self.shipPos then return maxSpeed end

    -- Find the strongest influence (closest body relative to its zone radius)
    for _, zone in ipairs(self.zones) do
        local parentRbCmp = zone.parentEntity:get(PhysicsComponents.RigidBody)
        if parentRbCmp then
            local parentRb = parentRbCmp:getRigidBody()
            if parentRb then
                local dist = self.shipPos:distance(parentRb:getPos())
                local radius = zone.radius

                -- Influence extends to 2x zone radius (smooth approach)
                local influenceRadius = radius * 2.0
                if dist < influenceRadius then
                    -- t: 0 at body center, 1 at edge of influence
                    local t = Math.Clamp(dist / influenceRadius, 0, 1)

                    -- Smooth interpolation: zone min speed at center → open space at edge
                    local zoneMin = cfg[zone.zoneType] or cfg.openSpace
                    local zoneSpeed = zoneMin + (cfg.openSpace - zoneMin) * t * t

                    -- Take the most restrictive (lowest) speed
                    if zoneSpeed < maxSpeed then
                        maxSpeed = zoneSpeed
                    end
                end
            end
        end
    end

    return maxSpeed
end

return GravityWellSystem()
