local Entity = require("Core.ECS.Entity")
local Components = require("Components")

---@return Entity
local function Camera()
    return Entity(
        "Camera",
        Components.RigidBodyComponent(),
        Components.TransformComponent(),
        Components.CameraData()
    )
end

return Camera
