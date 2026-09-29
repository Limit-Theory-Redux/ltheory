local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@param seed integer
---@param config table
---@return Entity
return function(seed, config)
    config = config or {}
    local pointLight = config.pointLight or {}
    local entity = Entity.Create(
        "PointLightEffectEntity",
        Core.Seed(seed or 0),
        Physics.Transform(),
        Rendering.PointLight(
            pointLight.color or config.color or Vec3f(),
            pointLight.radius or config.radius or 0,
            pointLight.intensity or config.intensity or 1),
        Rendering.LightEffect(
            config.duration or pointLight.duration or 0,
            config.fadeOutDuration or pointLight.fadeOutDuration,
            config.kind or "transient"))

    local transform = entity:get(Physics.Transform)
    transform:setPos(config.position or Position())
    return entity
end
