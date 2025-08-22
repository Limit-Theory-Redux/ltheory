local Entity = require("Core.ECS.Entity")

---@class TriggerEntity: Entity
---@return Entity
return function()
    local Core = require("Modules.Core")
    local Physics = require("Modules.Physics")
    local Spatial = require("Modules.Spatial")

    return Entity(
        "TriggerEntity",
        Physics.Components.Transform(),
        Spatial.Components.Shape(),
        Core.Components.Hierarchy()
    )
end
