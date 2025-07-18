local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Core", "Physics", "Spatial")

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
