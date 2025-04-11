local Entity = require("Entities.Entity")
local Components = require("Components")

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
