local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics")

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
