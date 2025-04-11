local Entity = require("Entities.Entity")
local Components = require("Components")

---@return Entity
local function ZoneEntity()
    return Entity(
        "ZoneEntity",
        Components.NameComponent(),
        Components.TransformComponent(),
        Components.ShapeComponent(),
        Components.HierarchyComponent()
    )
end

return ZoneEntity
