local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics", "Rendering")

---@param material Material
---@return Entity
local function BoxEntity(material)
    return Entity(
        "BoxEntity",
        Components.TransformComponent(),
        Components.RenderComponent({ material }, Enums.MeshType.Box),
        Components.RigidBodyComponent(),
        Components.HierarchyComponent()
    )
end

return BoxEntity
