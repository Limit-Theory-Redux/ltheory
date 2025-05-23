local Entity = require("Entities.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function UniverseEntity(seed)
    return Entity(
        "UniverseEntity",
        Components.SeedComponent(seed),
        Components.HierarchyComponent()
    )
end

return UniverseEntity
