local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics", "Rendering")
local Materials = require("Shared.Registries.Materials")

---@param seed integer
---@return Entity
local function AsteroidEntity(seed)
    return Entity(
        "AsteroidEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.RenderComponent({ Materials.Asteroid }, Enums.MeshType.Asteroid),
        Components.RigidBodyComponent(),
        Components.HierarchyComponent()
    )
end

return AsteroidEntity
