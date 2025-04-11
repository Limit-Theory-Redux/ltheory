local Entity = require("Entities.Entity")
local Components = require("Components")

---@param entityId integer
---@return Entity
local function ZoneEntity(entityId)
    return Entity(
        Components.NameComponent(),
        Components.TransformComponent(),
        Components.ShapeComponent(),
        Components.HierarchyComponent(entityId)
    )
end

return ZoneEntity
