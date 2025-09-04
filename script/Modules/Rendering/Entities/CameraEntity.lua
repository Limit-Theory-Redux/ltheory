local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@class CameraEntity: Entity
---@return CameraEntity
return function()
    return Entity(
        "CameraEntity",
        Physics.RigidBody(),
        Physics.Transform(),
        Rendering.CameraData()
    )
end
