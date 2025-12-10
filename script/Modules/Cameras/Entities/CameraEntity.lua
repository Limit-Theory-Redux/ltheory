local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Cameras = require("Modules.Cameras.Components")

---@return Entity
return function()
    return Entity.Create(
        "CameraEntity",
        Physics.RigidBody(),
        Physics.Transform(),
        Cameras.CameraData()
    )
end
