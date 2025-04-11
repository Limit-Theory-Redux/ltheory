local Entity = require("Entities.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function AsteroidRingEntity(seed)
    return Entity(
        "AsteroidRingEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return AsteroidRingEntity
