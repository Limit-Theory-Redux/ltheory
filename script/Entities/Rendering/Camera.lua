local Entity = require("Entities.Entity")

local RigidBody = require("Components.Physics.RigidBodyComponent")
local Transform = require("Components.Physics.TransformComponent")
local CameraData = require("Components.Rendering.CameraData")

---@class Camera: Entity
---@overload fun(self: Camera): Camera subclass internal
---@overload fun(): Camera subclass external
local Camera = Subclass("Camera", Entity, function(self)
    self:setArchetype(Enums.EntityArchetype.CameraEntity)

    -- PhysicsRigidBodyComponent
    self:addComponent(RigidBody())
    -- PhysicsTransformComponent
    self:addComponent(Transform())
    ---- CameraDataComponent
    self:addComponent(CameraData())
end)

return Camera
