local Entity = require("Entities.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function StarSystemEntity(seed)
    return Entity(
        Components.NameComponent(),
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return StarSystemEntity
