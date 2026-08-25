local Entity        = require("Core.ECS.Entity")
local Core          = require("Modules.Core.Components")
local Physics       = require("Modules.Physics.Components")
local Rendering     = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")

local ExplosionEffectComponent =
    require("Modules.Constructs.Components.ExplosionEffectComponent")
local Primitive = require("Legacy.Systems.Gen.Primitive")

---Volumetric fireball: procedural raymarch on a view-aligned billboard
---(vertex/billboard/quadpos.glsl x fragment/effect/explosion_volume.glsl).
---No textures/assets; density is fbm noise inside an expanding sphere.
local explosionMesh
local explosionShader

local function ensureAssets()
    if not explosionMesh then
        explosionMesh = Primitive.Billboard(-1, -1, 1, 1)
        explosionShader = Cache.Shader("billboard/explosion", "effect/explosion_volume")
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

    explosionShader:start()
    explosionShader:setFloat3("origin", pos.x - eye.x, pos.y - eye.y, pos.z - eye.z)
    explosionShader:setFloat("size", config.size)
    explosionShader:setFloat3("up", camUp.x, camUp.y, camUp.z)
    explosionShader:setFloat("age", config.age)
    explosionShader:setFloat("life", config.duration)
    explosionShader:setFloat("seed", config.seed % 1024)
    explosionMesh:draw()
    explosionShader:stop()
end

---Transient death/impact explosion: raymarched volumetric fireball plus an
---illuminating point light. Aging, looping and teardown are owned by
---ExplosionEffectSystem (call update(dt) once per frame); spawn-and-forget.
---@param seed integer
---@param config table {position, size, duration, loop?, lightColor=Color,
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
            lightColor = config.lightColor,
            lightIntensity = config.lightIntensity,
            lightRadius = config.lightRadius,
        }),
        Rendering.Render(render))

    local transform = entity:get(Physics.Transform)
    transform:setPos(config.position or Position())
    return entity
end
