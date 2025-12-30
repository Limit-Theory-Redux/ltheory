local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@param color Vec3f? Light color (default: white)
---@param intensity number? Light intensity (default: 1.0)
---@param radius number? Light radius (default: 100.0)
---@return Entity
return function(color, intensity, radius)
    return Entity.Create(
        "PointLightEntity",
        Physics.Transform(),
        Rendering.PointLight(color, intensity, radius)
    )
end
