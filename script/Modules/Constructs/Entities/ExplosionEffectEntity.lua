local Entity        = require("Core.ECS.Entity")
local Core          = require("Modules.Core.Components")
local Physics       = require("Modules.Physics.Components")
local Rendering     = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local Pipelines     = require("Render.Pipelines")

local ExplosionEffectComponent =
    require("Modules.Constructs.Components.ExplosionEffectComponent")
local Primitive = require("Legacy.Systems.Gen.Primitive")

---Volumetric fireball: procedural raymarch on a view-aligned billboard
---(vertex/billboard/explosion_draw.glsl x fragment/effect/explosion_volume_draw.glsl).
---No textures/assets; density is fbm noise inside an expanding sphere.
local explosionMesh
local explosionShader
local DrawBlock

local function ensureAssets()
    if not explosionMesh then
        explosionMesh = Primitive.Billboard(-1, -1, 1, 1)
        explosionShader = Cache.Shader("billboard/explosion_draw", "effect/explosion_volume_draw")
        DrawBlock = explosionShader:blockType("DrawBlock")
    end
end

---Render callback invoked by RenderCoreSystem during the additive pass.
---Positions are camera-relative at the draw boundary; the shader marches a
---ray per fragment through the explosion's bounding sphere.
local function render(entity, blendMode)
    if blendMode ~= BlendMode.Additive then
        return
    end

    local config = entity:get(ExplosionEffectComponent)
    if not config or config.age >= config.duration then
        return
    end
    if not config.renderProven then
        config.renderProven = true
        Log.Debug("[ExplosionEffect] render fn executed (first call)")
    end

    local transform = entity:get(Physics.Transform)
    local pos = transform and transform:getPos()
    if not pos then
        return
    end

    -- Billboard orientation from the active modern camera.
    local camUp = Vec3f(0, 1, 0)
    local camEntity = CameraManager:getActiveCameraEntity()
    if camEntity then
        local camTransform = camEntity:get(Physics.Transform)
        if camTransform then
            local rot = camTransform:getRot()
            if rot and rot.getUp then
                camUp = rot:getUp()
            end
        end
    end

    local eye = CameraManager:getEye()
    ensureAssets()

    -- One draw in the additive pass: the pipeline sets the state, the draw
    -- block (group 2) carries the shader's parameters (slots in the shaders).
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(explosionShader, Pipelines.Additive))
    local d = pass:alloc(DrawBlock)
    local m = d.mWorld
    m[0], m[5], m[10], m[15] = 1, 1, 1, 1
    local user = d.drawUser
    user[0], user[1], user[2] = config.age, config.seed % 1024, config.duration -- age, seed, life
    user[4], user[5], user[6] = pos.x - eye.x, pos.y - eye.y, pos.z - eye.z -- origin (camera-relative)
    user[7] = config.size
    user[8], user[9], user[10] = camUp.x, camUp.y, camUp.z -- up
    pass:drawMesh(explosionMesh)
end

---Transient death/impact explosion: raymarched volumetric fireball plus an
---illuminating point light. Aging, looping and teardown are owned by
---ExplosionEffectSystem (call update(dt) once per frame); spawn-and-forget.
---@param seed integer
---@param config table {position, size, duration, loop?, flare?, lightColor=Color,
--- lightIntensity, lightRadius}
---@return Entity
return function(seed, config)
    config = config or {}

    local entity = Entity.Create(
        "ExplosionEffectEntity",
        Core.Seed(seed or 0),
        Physics.Transform(),
        Rendering.PointLight(
            config.lightColor or Color(1.0, 0.55, 0.25, 1.0),
            config.lightRadius or 1.0,
            config.lightIntensity or 3.0),
        ExplosionEffectComponent({
            seed = seed or 0,
            size = config.size,
            duration = config.duration,
            loop = config.loop,
            flare = config.flare,
            lightColor = config.lightColor,
            lightIntensity = config.lightIntensity,
            lightRadius = config.lightRadius,
        }),
        Rendering.Render(render))

    local transform = entity:get(Physics.Transform)
    transform:setPos(config.position or Position())
    return entity
end
