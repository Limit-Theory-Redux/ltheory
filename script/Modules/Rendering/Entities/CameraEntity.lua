local Entity = require("Core.ECS.Entity")

---@class CameraEntity: Entity
---@return Entity
return function()
    local Physics = require("Modules.Physics")
    local Rendering = require("Modules.Rendering")

    return Entity(
        "CameraEntity",
        Physics.Components.RigidBody(),
        Physics.Components.Transform(),
        Rendering.Components.CameraData()
    )
end
