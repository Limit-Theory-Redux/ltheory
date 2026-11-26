local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Spatial = require("Modules.Spatial.Components")

---@return Entity
return function()
    return Entity.Create(
        "ZoneEntity",
        Spatial.Zone(),
        Physics.Transform(),
        Spatial.Shape()
    )
end
