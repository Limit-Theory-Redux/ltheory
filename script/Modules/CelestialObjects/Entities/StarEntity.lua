local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics")

---@param seed integer
---@return Entity
local function StarEntity(seed)
    return Entity(
        "StarEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return StarEntity
