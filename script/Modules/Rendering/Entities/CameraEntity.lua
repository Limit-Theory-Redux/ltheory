local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@return Entity
return function()
    return Entity.Create(
        "CameraEntity",
        Physics.RigidBody(),
        Physics.Transform(),
        Rendering.CameraData()
    )
end
