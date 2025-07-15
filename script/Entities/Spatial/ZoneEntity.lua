local Entity = require("Core.ECS.Entity")
local Components = require("Components")

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
