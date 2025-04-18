local Entity = require("Entities.Entity")
local Components = require("Components")

local function AsteroidBeltEntity(seed)
    return Entity(
        "AsteroidBeltEntity",
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent()
    )
end

return AsteroidBeltEntity
