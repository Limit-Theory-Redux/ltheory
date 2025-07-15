local Entity = require("Core.ECS.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function MoonEntity(seed)
    return Entity(
        "MoonEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return MoonEntity
