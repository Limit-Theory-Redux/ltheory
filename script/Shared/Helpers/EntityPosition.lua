local PhysicsComponents = require("Modules.Physics.Components")

---Where an entity is: its rigid body when it has one, else its transform.
---@class EntityPosition
local EntityPosition = {}

---The current position of an entity, the origin when it has neither a rigid body nor a transform.
---@param entity Entity|nil
---@return Position
function EntityPosition.Get(entity)
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

return EntityPosition
