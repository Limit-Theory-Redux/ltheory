local Entity = require("Entities.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function PlanetEntity(seed)
    return Entity(
        "PlanetEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return PlanetEntity
