local Entity = require("Entities.Entity")
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
