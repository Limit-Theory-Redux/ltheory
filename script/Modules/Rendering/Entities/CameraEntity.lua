local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics", "Rendering")

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
