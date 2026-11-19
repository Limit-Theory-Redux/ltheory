-- The entities whose bodies lie near a ship, found through the physics query index that avoidance
-- senses through. Design: doc/script/nav/navigation.md

local Registry          = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")

---@class NearbyBodies
local NearbyBodies = {
    -- The collision mask that matches every collision group.
    ALL_GROUPS = 0xFFFFFFFF,
}

---One body near a ship.
---@class NavNeighbour
---@field entity Entity
---@field distance gameunit From the ship's centre to the neighbour's surface

---The entities whose bodies lie within `radius` of the ship's centre, nearest first. The ship itself is
---left out, and so is a body with no entity. The positions are those of the last physics update.
---@param physics Physics
---@param ship Entity
---@param radius gameunit
---@param filter (fun(entity: Entity, distance: gameunit): boolean)|nil Keeps the neighbours it returns true for
---@return NavNeighbour[]
function NearbyBodies.of(physics, ship, radius, filter)
    local rb = ship:get(PhysicsComponents.RigidBody)
    assert(rb, "NearbyBodies: the ship has no rigid body")
    rb = rb:getRigidBody()
    local found = physics:queryRadius(rb:getPos(), radius, rb, NearbyBodies.ALL_GROUPS)
    local neighbours, seen = {}, {}
    for i = 0, found.len - 1 do
        local id = tonumber(found.ids[i])
        -- A body with several colliders is listed once for each; the nearest comes first.
        if id ~= 0 and not seen[id] then
            seen[id] = true
            local entity = Registry:getEntityById(id)
            local distance = found.distances[i]
            if entity and entity.id ~= ship.id and (not filter or filter(entity, distance)) then
                neighbours[#neighbours + 1] = { entity = entity, distance = distance }
            end
        end
    end
    return neighbours
end

---Whether any body, with an entity or not, lies within `radius` of the ship's centre, the ship's own
---excluded. It asks what avoidance's sensing would find, without building the list.
---@param physics Physics
---@param ship Entity
---@param radius gameunit
---@return boolean
function NearbyBodies.anyWithin(physics, ship, radius)
    local rb = ship:get(PhysicsComponents.RigidBody)
    assert(rb, "NearbyBodies: the ship has no rigid body")
    rb = rb:getRigidBody()
    return physics:queryRadius(rb:getPos(), radius, rb, NearbyBodies.ALL_GROUPS).len > 0
end

return NearbyBodies
