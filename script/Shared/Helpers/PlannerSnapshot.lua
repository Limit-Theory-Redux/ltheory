-- A snapshot of the star system for the system planner: every star, planet, moon and station as a
-- body of the planner's body buffer (orbits at the moment of the snapshot, hull and safe radii,
-- travel-drive zones), the cost zones weighted by a cost profile, and the frames that place a
-- plan's waypoints as the bodies move on.
-- Design: doc/script/nav/system-planner.md

local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local OrbitalSystem       = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local EntityPosition      = require("Shared.Helpers.EntityPosition")
local SystemPlanner       = require("Shared.Helpers.SystemPlanner")
local CostZones           = require("Shared.Helpers.CostZones")

local Body      = SystemPlanner.Body
local NO_PARENT = -1

-- The kinds of body, as the keys of `Config.game.autoPilot.safeRadiusFactor`; a body that orbits
-- or is fixed in place is found by its marker component.
local BODY_KINDS = {
    { marker = CelestialComponents.Star,         kind = "star" },
    { marker = CelestialComponents.Planet,       kind = "planet" },
    { marker = CelestialComponents.Moon,         kind = "moon" },
    { marker = ConstructComponents.SpaceStation, kind = "station" },
}

---@class PlannerSnapshot
---@field bodies number[] The planner's body buffer, `count * BODY_STRIDE` numbers
---@field count integer
---@field entities Entity[] The entity of each body, 1-based like a plan's `frame`
---@field frames PlanFrame[] A frame per body, for `PlanFollower`
---@field index table<EntityId, integer> The 1-based body index of an entity
---@field zones number[] The planner's zone buffer, `zoneCount * ZONE_STRIDE` numbers, costs weighted by the profile
---@field zoneCount integer
---@field age number Seconds since the snapshot was taken
local PlannerSnapshot = {}
PlannerSnapshot.__index = PlannerSnapshot

---@param entity Entity
---@return Entity|nil
local function parentOf(entity)
    local parentCmp = entity:get(CoreComponents.Parent)
    return parentCmp and parentCmp:getParent() or nil
end

---The bodies of the system, each with its kind, in no order.
---@return table<EntityId, { entity: Entity, kind: string }>
local function collectBodies()
    local found = {}
    for _, spec in ipairs(BODY_KINDS) do
        for entity in Registry:view(spec.marker) do
            local rbCmp = entity:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then
                found[entity.id] = { entity = entity, kind = spec.kind }
            end
        end
    end
    return found
end

---The bodies in an order that puts a parent before its children.
---@param found table<EntityId, { entity: Entity, kind: string }>
---@return { entity: Entity, kind: string }[]
local function parentsFirst(found)
    local ordered, seen = {}, {}
    local function visit(item)
        if seen[item.entity.id] then return end
        seen[item.entity.id] = true
        local parent = parentOf(item.entity)
        if parent and found[parent.id] then visit(found[parent.id]) end
        ordered[#ordered + 1] = item
    end
    -- Visit in id order so that the same system gives the same snapshot.
    local ids = {}
    for id in pairs(found) do ids[#ids + 1] = id end
    table.sort(ids)
    for _, id in ipairs(ids) do visit(found[id]) end
    return ordered
end

---The zone each body declares, by body id.
---@return table<EntityId, table>
local function zonesByBody()
    local zones = {}
    for _, zone in ipairs(GravityWellSystem.zones) do zones[zone.zoneEntity.id] = zone end
    return zones
end

---Where a body is not on an orbit that the snapshot holds, it stays: its own position, or, for an
---orbit around a body outside the snapshot, that parent's.
---@param entity Entity
---@param orbit OrbitComponent|nil
---@param parent Entity|nil
---@return Position
local function anchorOf(entity, orbit, parent)
    if orbit and parent then return EntityPosition.Get(parent) end
    return EntityPosition.Get(entity)
end

---Writes the body row of `item` into `values`.
---@param values number[]
---@param item { entity: Entity, kind: string }
---@param indexOf table<EntityId, integer>
---@param zone table|nil
local function appendBody(values, item, indexOf, zone)
    local entity = item.entity
    local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    -- The orbit the scene runs; a body whose orbit is not run is a fixed one, as it is for the prediction.
    local orbit = OrbitalSystem:runningOrbit(entity)
    local parent = parentOf(entity)
    local parentIndex = orbit and parent and indexOf[parent.id]
    local outsideParent = not parentIndex and parent or nil
    local at = anchorOf(entity, orbit, outsideParent)
    local drive = Config.game.travelDrive.maxSpeedByZone
    local radius = rb:getBoundingRadius()

    local row = {}
    row[Body.Parent + 1] = parentIndex and parentIndex - 1 or NO_PARENT
    row[Body.X + 1], row[Body.Y + 1], row[Body.Z + 1] = at.x, at.y, at.z
    row[Body.OrbitRadius + 1] = orbit and orbit:getOrbitRadius() or 0
    row[Body.Phase + 1] = orbit and orbit:getPhase() or 0
    row[Body.AngularSpeed + 1] = orbit and orbit:getAngularSpeed() or 0
    row[Body.Inclination + 1] = orbit and orbit:getInclination() or 0
    row[Body.Radius + 1] = radius
    row[Body.SafeRadius + 1] = radius * Config.game.autoPilot.safeRadiusFactor[item.kind]
    row[Body.ZoneRadius + 1] = zone and zone.radius or 0
    row[Body.ZoneCap + 1] = zone and (drive[zone.zoneType] or drive.openSpace) or 0
    for i = 1, SystemPlanner.BODY_STRIDE do values[#values + 1] = row[i] end
end

---Takes a snapshot of the system as it is now.
---@param costProfile CostProfile|nil How the ship weighs the cost zones; nil takes each at its path cost
---@return PlannerSnapshot
function PlannerSnapshot.take(costProfile)
    local self = setmetatable({ bodies = {}, entities = {}, frames = {}, index = {}, age = 0 }, PlannerSnapshot)
    local ordered = parentsFirst(collectBodies())
    for i, item in ipairs(ordered) do self.index[item.entity.id] = i end
    local zones = zonesByBody()
    for i, item in ipairs(ordered) do
        appendBody(self.bodies, item, self.index, zones[item.entity.id])
        self.entities[i] = item.entity
        local entity = item.entity
        self.frames[i] = {
            rb = entity:get(PhysicsComponents.RigidBody):getRigidBody(),
            -- The entity's orbit runs on from now; the snapshot's time zero was `age` seconds ago.
            positionAt = function(t) return OrbitalSystem:predictPos(entity, t - self.age) end,
        }
    end
    self.count = #ordered
    self.zones, self.zoneCount = CostZones.collect(costProfile, function(entity) return self.index[entity.id] end)
    return self
end

---Lets time pass for the frames.
---@param dt number Seconds
function PlannerSnapshot:advance(dt)
    self.age = self.age + dt
end

---Whether an entity is a body of the planner's snapshots: a plan to it rides on it, so the plan's
---waypoints follow it as it moves.
---@param entity Entity
---@return boolean
function PlannerSnapshot.isBody(entity)
    for _, spec in ipairs(BODY_KINDS) do
        if entity:get(spec.marker) then
            local rbCmp = entity:get(PhysicsComponents.RigidBody)
            return rbCmp ~= nil and rbCmp:getRigidBody() ~= nil
        end
    end
    return false
end

---The 1-based body index of an entity, nil when it isn't a body of the snapshot.
---@param entity Entity
---@return integer|nil
function PlannerSnapshot:indexOf(entity)
    return self.index[entity.id]
end

return PlannerSnapshot
