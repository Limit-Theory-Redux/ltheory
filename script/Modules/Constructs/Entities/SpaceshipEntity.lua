local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics")

---@param seed integer
---@return Entity
local function SpaceshipEntity(seed)
    return Entity(
        "SpaceshipEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return SpaceshipEntity
