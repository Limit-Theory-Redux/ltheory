local Entity = require("Core.ECS.Entity")
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
