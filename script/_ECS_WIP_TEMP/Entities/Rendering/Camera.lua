local Entity = require("_ECS_WIP_TEMP.Entity")

local RigidBody = require("Components.Physics.RigidBody")
local Transform = require("Components.Physics.Transform")

---@class Camera: Entity
---@overload fun(self: Camera, name: string) subclass interal
---@overload fun(name: string) subclass external
local Camera = Subclass(Entity, function(self, name)

    self:setArchetype(Enums.EntityArchetype.Camera)

    -- PhysicsRigidBodyComponent
    self:addComponent(RigidBody())
    -- PhysicsTransformComponent
    self:addComponent(Transform())
end)

return Camera
