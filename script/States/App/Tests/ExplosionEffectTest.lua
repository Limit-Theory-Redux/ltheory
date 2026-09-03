local Application      = require('States.Application')

---@class ExplosionEffectTest: Application
local ExplosionEffectTest = Subclass("ExplosionEffectTest", Application)

local Registry         = require("Core.ECS.Registry")
local CoreComponents   = require('Modules.Core.Components')
local Rendering        = require("Modules.Rendering.Components")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local LightManager     = require("Modules.Rendering.Managers.LightManager")
local ExplosionEffectSystem = require("Modules.Constructs.Systems.ExplosionEffectSystem")
local ExplosionEffectComponent = require("Modules.Constructs.Components.ExplosionEffectComponent")
local CameraEntity     = require("Modules.Cameras.Entities").Camera
local CameraDataComponent = require('Modules.Cameras.Components.CameraDataComponent')
local OrbitCameraController = require('Modules.Cameras.Managers.CameraControllers.OrbitCameraController')
local SkyboxEntity     = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraManager    = require("Modules.Cameras.Managers.CameraManager")

require('Shared.Definitions.MaterialDefs')
require('Shared.Definitions.UniformFuncDefs')

local DURATION   = 3.0   --- one-shot explosion lifetime under test (seconds)
local LOOP_DURATION = 1.2 --- looping explosion cycle length (seconds)
local PHASE_WAIT = 1.0   --- grace seconds before teardown assertions

local rng = RNG.Create(0xE0C1)
local explosionSystem = ExplosionEffectSystem()

function ExplosionEffectTest:onInit()
    -- Environment bootstrap (established SkyboxEntity nebula/starfield pattern).
    self.skybox = SkyboxEntity(rng:get31(), function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)
        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
        end
        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local nebulaRNG     = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle     = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            local Generator     = require("Legacy.Systems.Gen.Generator")
            local Starfield     = require("Legacy.Systems.Gen.Starfield")
            placeholder.envMap  = Generator.Get('Nebula', nebulaRNG)(nebulaRNG, Config.gen.nebulaRes, placeholder.starDir)
            placeholder.irMap   = placeholder.envMap:genIRMap(256)
            placeholder.stars   = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            CameraManager:setStarDir(placeholder.starDir)
            ShaderVar.PushTexCube('envMap', placeholder.envMap)
            ShaderVar.PushTexCube('irMap', placeholder.irMap)
        end
        if blendMode == BlendMode.Disabled then
            RenderState.PushDepthWritable(false)
            local shader = Cache.Shader('farplane', 'skybox')
            shader:start()
            Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
            shader:stop()
            RenderState.PopDepthWritable()
        elseif blendMode == BlendMode.Additive then
            local shader = Cache.Shader('farplane', 'starbg')
            shader:start()
            shader:setFloat('brightnessScale', 3)
            shader:setTexCube('irMap', placeholder.irMap)
            shader:setTexCube('envMap', placeholder.envMap)
            placeholder.stars:draw()
            shader:stop()
        end
    end)

    -- Camera: orbit around the origin where the explosions fire.
    local cam = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam)
    self.controllerOrbitCam = OrbitCameraController(cam)
    self.controllerOrbitCam:setTarget(nil)
    self.controllerOrbitCam:setDistance(60)
    cam:get(CameraDataComponent):setController(self.controllerOrbitCam)
    CameraManager:setActiveCamera("OrbitCam")

    self.elapsed = 0.0
    self.spawnDone = false
    self.renderVerdictSent = false
    self.loopVerdictSent = false
    self.teardownVerdictSent = false

    self.explosionSystem = ExplosionEffectSystem()
end

---@param data EventData
function ExplosionEffectTest:onSim(data)
    local dt = data:deltaTime()

    if not self.spawnDone then
        self.spawnDone = true
        local ExplosionEffect =
            require("Modules.Constructs.Entities.ExplosionEffectEntity")

        -- One-shot effect under test.
        self.entity = ExplosionEffect(rng:get31(), {
            position = Position(0, 0, 0),
            size = 12.0,
            duration = DURATION,
            lightIntensity = 4.0,
            lightRadius = 2.5,
        })

        -- Looping effect beside it.
        self.loopEntity = ExplosionEffect(rng:get31(), {
            position = Position(25, 0, 0),
            size = 6.0,
            duration = LOOP_DURATION,
            loop = true,
        })

        -- Contract 1: data-only components carry seed/lifetime/light config.
        local comp = self.entity:get(ExplosionEffectComponent)
        assert(comp ~= nil, "[ExplosionEffectTest] FAIL: component missing")
        assert(math.abs(comp.duration - DURATION) < 1e-6,
            "[ExplosionEffectTest] FAIL: duration not applied")
        assert(comp.loop == false, "[ExplosionEffectTest] FAIL: default must be non-looping")
        local lcfg = self.loopEntity:get(
            require("Modules.Constructs.Components.ExplosionEffectComponent"))
        assert(lcfg.loop == true, "[ExplosionEffectTest] FAIL: loop flag not applied")
        local light = self.entity:get(Rendering.PointLight)
        assert(light ~= nil, "[ExplosionEffectTest] FAIL: point light missing")
        assert(light.color.r ~= nil and light.color.a ~= nil,
            "[ExplosionEffectTest] FAIL: light color must be engine Color type")
        Log.Info("[ExplosionEffectTest] spawn contract PASS")
    end

    -- System under test drives aging/looping/teardown.
    self.explosionSystem:update(dt)
    self.elapsed = self.elapsed + dt

    -- Contract 1b: while effects live, their lights appear in the render
    -- core snapshot; shader compile proof is the ShaderWatcher
    -- 'billboard:effect/explosion_volume' registration line in the log.
    if not self.renderVerdictSent and self.elapsed > 1.5 then
        self.renderVerdictSent = true
        local lights = LightManager:getPointLights()
        assert(#lights >= 2,
            "[ExplosionEffectTest] FAIL: effect lights absent from "
            .. "LightManager snapshot (shader compile failure or early teardown)")
        Log.Info(string.format(
            "[ExplosionEffectTest] render execution PASS (%d live effect lights)",
            #lights))
    end

    -- Contract 2: looping effect survives past its first cycle with age
    -- wrapped (restart without respawn).
    if not self.loopVerdictSent and self.elapsed > LOOP_DURATION + PHASE_WAIT then
        self.loopVerdictSent = true
        local lcfg = self.loopEntity:get(ExplosionEffectComponent)
        assert(self.loopEntity:isValid(),
            "[ExplosionEffectTest] FAIL: looping entity was destroyed")
        assert(lcfg.age < LOOP_DURATION,
            "[ExplosionEffectTest] FAIL: loop age did not wrap (age="
            .. tostring(lcfg.age) .. ")")
        Log.Info("[ExplosionEffectTest] loop restart PASS")
    end

    -- Contract 3: the one-shot effect is destroyed after its duration.
    if not self.teardownVerdictSent and self.elapsed > DURATION + PHASE_WAIT then
        self.teardownVerdictSent = true
        assert(not self.entity:isValid(),
            "[ExplosionEffectTest] FAIL: one-shot entity outlived its duration")
        Log.Info("[ExplosionEffectTest] one-shot teardown PASS")
        Log.Info("[ExplosionEffectTest] ALL PASS")
        self:quit()
    end
end

---@param data EventData
function ExplosionEffectTest:onRender(data)
    RenderCoreSystem:render(data)
end

return ExplosionEffectTest
