local Entity = require("Core.ECS.Entity")

---@class ZoneEntity: Entity
---@return Entity
return function()
    local Core = require("Modules.Core")
    local Physics = require("Modules.Physics")
    local Spatial = require("Modules.Spatial")

    return Entity(
        "ZoneEntity",
        Physics.Components.Transform(),
        Spatial.Components.Shape(),
        Core.Components.Hierarchy()
    )
end
