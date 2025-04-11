local Entity = require("Entities.Entity")
local Components = require("Components")

---@param entityId integer
---@return Entity
local function TriggerEntity(entityId)
    return Entity(
        Components.NameComponent(),
        Components.TransformComponent(),
        Components.ShapeComponent(),
        Components.HierarchyComponent(entityId)
    )
end

return TriggerEntity
