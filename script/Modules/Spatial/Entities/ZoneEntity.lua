local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics", "Spatial")

---@return Entity
local function ZoneEntity()
    return Entity(
        "ZoneEntity",
        Components.TransformComponent(),
        Components.ShapeComponent(),
        Components.HierarchyComponent()
    )
end

return ZoneEntity
