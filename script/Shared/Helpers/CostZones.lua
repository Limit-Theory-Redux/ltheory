-- The cost zones of the system as the planner's zone buffer: every zone that carries a path cost,
-- with its cost multiplied by the weight a cost profile gives its tags.
-- Design: doc/script/system-planner.md

local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local SystemPlanner       = require("Shared.Helpers.SystemPlanner")

local Zone = SystemPlanner.Zone

---@class CostZones
local CostZones = {}

---The zones that carry a path cost, in id order so that the same system gives the same buffer.
---@return Entity[]
local function costZones()
    local found = {}
    for entity in Registry:view(NavigationComponents.PathCost) do
        if entity:get(SpatialComponents.Zone) then found[#found + 1] = entity end
    end
    table.sort(found, function(a, b) return a.id < b.id end)
    return found
end

---The zone's sphere, as the zone row's centre: a zone that is attached to a body of the system
---rides on it, with no offset (as a body's gravity well does), and any other zone is a point of
---the system.
---@param zone Entity
---@param indexOf fun(entity: Entity): integer|nil The 1-based body index of an entity of the snapshot
---@return integer frame 0-based body index, or `SystemPlanner.SYSTEM_FRAME`
---@return number x
---@return number y
---@return number z
local function centreOf(zone, indexOf)
    local parentCmp = zone:get(CoreComponents.Parent)
    local parent = parentCmp and parentCmp:getParent()
    local bodyIndex = parent and indexOf(parent)
    if bodyIndex then return bodyIndex - 1, 0, 0, 0 end
    local transform = zone:get(PhysicsComponents.Transform)
    assert(transform, "CostZones: a zone with a path cost needs a body to ride on or a transform")
    local at = transform:getPos()
    return SystemPlanner.SYSTEM_FRAME, at.x, at.y, at.z
end

---The zone's radius: only a sphere can be planned around.
---@param zone Entity
---@return gameunit
local function radiusOf(zone)
    local shapeCmp = zone:get(SpatialComponents.Shape)
    assert(shapeCmp and shapeCmp:getShape() == Enums.ZoneShape.Sphere,
        "CostZones: a zone with a path cost must be a sphere, for now")
    local radius = shapeCmp:getRadius()
    assert(type(radius) == "number" and radius > 0, "CostZones: a zone's radius must be above zero")
    return radius
end

---The zone buffer for a request: a row of `SystemPlanner.ZONE_STRIDE` numbers per zone that has
---a path cost. The cost of a row is the zone's path cost times the profile's weight of its tags
---(`CostProfile:weightOf`); a ship with no profile takes every zone at its path cost.
---@param profile CostProfile|nil
---@param indexOf fun(entity: Entity): integer|nil The 1-based body index of an entity of the snapshot, nil for an entity that is not one
---@return number[] values `count * ZONE_STRIDE` numbers
---@return integer count
function CostZones.collect(profile, indexOf)
    local values, count = {}, 0
    for _, zone in ipairs(costZones()) do
        local frame, x, y, z = centreOf(zone, indexOf)
        local weight = profile and profile:weightOf(zone:get(CoreComponents.Tag)) or 1
        local row = {}
        row[Zone.Frame + 1] = frame
        row[Zone.X + 1], row[Zone.Y + 1], row[Zone.Z + 1] = x, y, z
        row[Zone.Radius + 1] = radiusOf(zone)
        row[Zone.Cost + 1] = zone:get(NavigationComponents.PathCost):getCost() * weight
        for i = 1, SystemPlanner.ZONE_STRIDE do values[#values + 1] = row[i] end
        count = count + 1
    end
    return values, count
end

return CostZones
