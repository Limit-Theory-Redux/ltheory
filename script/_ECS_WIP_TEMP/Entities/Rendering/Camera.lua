local Entity = require("_ECS_WIP_TEMP.Entities.Entity")                          --!temp path

local RigidBody = require("_ECS_WIP_TEMP.Components.Physics.RigidBodyComponent") --!temp path
local Transform = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local CameraData = require("_ECS_WIP_TEMP.Components.Rendering.CameraData")      --!temp path

---@class Camera: Entity
---@overload fun(self: Camera): Camera subclass internal
---@overload fun(): Camera subclass external
local Camera = Subclass(Entity, function(self)
    self:setArchetype(Enums.EntityArchetype.CameraEntity)

    -- PhysicsRigidBodyComponent
    self:addComponent(RigidBody())
    -- PhysicsTransformComponent
    self:addComponent(Transform())
    ---- CameraDataComponent
    self:addComponent(CameraData())
end)

return Camera
