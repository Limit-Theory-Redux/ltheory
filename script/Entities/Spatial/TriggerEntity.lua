local Entity = require("Entities.Entity")
local Components = require("Components")

---@return Entity
local function TriggerEntity()
    return Entity(
        "TriggerEntity",
        Components.TransformComponent(),
        Components.ShapeComponent(),
        Components.HierarchyComponent()
    )
end

return TriggerEntity
