local Entity = require("Core.ECS.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function StarSystemEntity(seed)
    return Entity(
        "StarSystemEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return StarSystemEntity
