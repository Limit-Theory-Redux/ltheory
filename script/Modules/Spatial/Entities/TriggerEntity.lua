local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Spatial = require("Modules.Spatial.Components")

---@class TriggerEntity: Entity
---@return Entity
return function()
    return Entity(
        "TriggerEntity",
        Physics.Transform(),
        Spatial.Shape(),
        Core.Hierarchy()
    )
end
