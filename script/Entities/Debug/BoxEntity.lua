local Entity = require("Entities.Entity")
local Components = require("Components")

---@param material Material
---@return Entity
local function BoxEntity(material)
    return Entity(
        Components.TransformComponent(),
        Components.RenderComponent({ material }, Enums.MeshType.Box),
        Components.RigidBodyComponent(),
        Components.HierarchyComponent()
    )
end

return BoxEntity
