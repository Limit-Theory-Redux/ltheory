local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local Pipelines = require("Render.Pipelines")

local impactMesh
local impactShader
local DrawBlock

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
        impactShader = Cache.Shader("billboard/quad_draw", "effect/pulsehead_draw")
        DrawBlock = impactShader:blockType("DrawBlock")
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
    -- One draw in the additive pass: the pipeline sets the state, the draw
    -- block (group 2) carries the transform and the sprite parameters.
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(impactShader, Pipelines.Additive))
    local d = pass:alloc(DrawBlock)
    local m = d.mWorld
    m[0], m[5], m[10], m[15] = 1, 1, 1, 1
    m[12], m[13], m[14] = pos.x - eye.x, pos.y - eye.y, pos.z - eye.z
    local user = d.drawUser
    user[0], user[1], user[2], user[3] = cr, cg, cb, fade * 0.9 -- color, alpha
    user[4] = (light:getRadius() > 0 and light:getRadius() or 0.2) * 4.0 -- size
    pass:drawMesh(impactMesh)
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
