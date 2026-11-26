-- The sphere of a zone entity: its radius, what it rides on and where its centre is. The planner's cost
-- zones and the zone events read the zone through here, so both mean the same sphere.
-- Design: doc/script/nav/navigation.md

local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local SpatialComponents = require("Modules.Spatial.Components")
local EntityPosition    = require("Shared.Helpers.EntityPosition")

---@class ZoneSphere
local ZoneSphere = {}

---The zone's radius, or nil when the zone is not a sphere with a radius above zero (only a sphere can
---be planned around or tested against).
---@param zone Entity
---@return gameunit|nil
function ZoneSphere.radius(zone)
    local shape = zone:get(SpatialComponents.Shape)
    if not shape or shape:getShape() ~= Enums.ZoneShape.Sphere then return nil end
    local radius = shape:getRadius()
    return type(radius) == "number" and radius > 0 and radius or nil
end

---The entity the zone rides on, with its centre at the entity's: the zone's parent. A zone with no
---parent is a fixed point of the system.
---@param zone Entity
---@return Entity|nil
function ZoneSphere.carrier(zone)
    local parentCmp = zone:get(CoreComponents.Parent)
    return parentCmp and parentCmp:getParent() or nil
end

---The zone's own position, where it is a fixed point of the system.
---@param zone Entity
---@return Position
function ZoneSphere.point(zone)
    local transform = zone:get(PhysicsComponents.Transform)
    assert(transform, "ZoneSphere: a zone needs a body to ride on or a transform")
    return transform:getPos()
end

---Where the zone's centre is now: at its carrier when that has a rigid body to tell where it is, else
---the zone's own point.
---@param zone Entity
---@return Position
function ZoneSphere.centre(zone)
    local carrier = ZoneSphere.carrier(zone)
    if carrier and carrier:get(PhysicsComponents.RigidBody) then return EntityPosition.Get(carrier) end
    return ZoneSphere.point(zone)
end

---@param zone Entity
---@param radius gameunit From `ZoneSphere.radius`
---@param position Position
---@return boolean
function ZoneSphere.contains(zone, radius, position)
    return ZoneSphere.centre(zone):distance(position) <= radius
end

return ZoneSphere
