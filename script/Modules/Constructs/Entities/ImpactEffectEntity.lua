local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")

local impactMesh
local impactShader

---Draw one billboard flash at the effect position. Color/intensity come
---from the damage source's impact definition; alpha fades over the
---LightEffect lifetime so the flash decays smoothly.
local function render(entity, blendMode)
    if blendMode ~= BlendMode.Additive then
        return
    end

    local light = entity:get(Rendering.PointLight)
    local effect = entity:get(Rendering.LightEffect)
    if not light or not effect or effect.remaining <= 0 then
        return
    end

    if not impactMesh then
        impactMesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
        impactShader = Cache.Shader("billboard/quad", "effect/pulsehead")
    end

    local transform = entity:get(Physics.Transform)
    local pos = transform and transform:getPos()
    if not pos then
        return
    end

    local eye = CameraManager:getEye()
    local fade = math.min(1, math.max(0,
        effect.remaining / math.max(effect.duration, 0.001)))

    local color = light:getColor()
    -- Colors may be Color (r/g/b) or Vec3f (x/y/z) depending on the def.
    local cr = color.r ~= nil and color.r or color.x
    local cg = color.g ~= nil and color.g or color.y
    local cb = color.b ~= nil and color.b or color.z
    impactShader:start()
    impactShader:setFloat3("color", cr, cg, cb)
    impactShader:setFloat("alpha", fade * 0.9)
    impactShader:setMatrix("mWorld",
        Matrix.Translation(pos.x - eye.x, pos.y - eye.y, pos.z - eye.z))
    impactShader:setFloat("size", (light:getRadius() > 0 and light:getRadius() or 0.2) * 4.0)
    impactMesh:draw()
    impactShader:stop()
end

---Transient impact effect at a hit position: colored light flash plus a
---billboard sprite whose color/size/duration come from the damage source's
---impact definition.
---@param seed integer
---@param config table {position, color, intensity, radius, duration}
---@return Entity
return function(seed, config)
    config = config or {}
    local duration = config.duration or 0.3
    local entity = Entity.Create(
        "ImpactEffectEntity",
        Core.Seed(seed or 0),
        Physics.Transform(),
        Rendering.PointLight(
            config.color or Vec3f(1, 1, 1),
            config.radius or 0.08,
            config.intensity or 1.5),
        Rendering.LightEffect(
            duration,
            duration,
            "transient"),
        Rendering.Render(render))

    local transform = entity:get(Physics.Transform)
    transform:setPos(config.position or Position())
    return entity
end
