local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Spatial = require("Modules.Spatial.Components")

---@return Entity
return function()
    return Entity.Create(
        "ZoneEntity",
        Physics.Transform(),
        Spatial.Shape(),
        Core.Hierarchy()
    )
end
