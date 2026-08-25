local Application      = require('States.Application')

local Registry         = require("Core.ECS.Registry")
local CoreComponents   = require('Modules.Core.Components')
local Physics          = require("Modules.Physics.Components")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local LightManager     = require("Modules.Rendering.Managers.LightManager")
local ExplosionEffectSystem = require("Modules.Constructs.Systems.ExplosionEffectSystem")
local ExplosionEffectComponent = require("Modules.Constructs.Components.ExplosionEffectComponent")
local CameraEntity     = require("Modules.Cameras.Entities").Camera
local CameraDataComponent = require('Modules.Cameras.Components.CameraDataComponent')
local OrbitCameraController = require('Modules.Cameras.Managers.CameraControllers.OrbitCameraController')
local SkyboxEntity     = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraManager    = require("Modules.Cameras.Managers.CameraManager")
-- Keep-alive: CameraSystem owns controller input/pre-render updates through
-- its event subscriptions (same convention as the weapon testbed).
local CameraSystem     = require("Modules.Cameras.Systems.CameraSystem")
local ExplosionEffect  = require("Modules.Constructs.Entities.ExplosionEffectEntity")
local LensFlareSystem  = require("Modules.Rendering.Systems.LensFlareSystem")
local Actions          = require("Input.ActionBindings.ExplosionTestbedActions")
local DrawEx           = require("UI.DrawEx")

require('Shared.Definitions.MaterialDefs')
require('Shared.Definitions.UniformFuncDefs')

local rng = RNG.Create(0xE0C2) ---@diagnostic disable-line: lowercase-global

---@class ExplosionTestbed: Application
---Sandbox for iterating on the volumetric explosion effect. Exactly ONE
---explosion lives at the origin so the orbiting camera always frames the
---subject: F respawns it, K toggles loop mode (persistent cycling fireball -
---ideal for shader tuning), T cycles scene-calibrated presets, Q/E scales
---size and Z/X scales duration live. Shader edits hot-reload via
---ShaderWatcher: tweak res/shader/fragment/effect/explosion_volume.glsl and
---watch the result immediately on the looping fireball.
local ExplosionTestbed = Subclass("ExplosionTestbed", Application)

-- Scene-calibrated presets (match the weapon testbed's deathExplosion
-- sizes so what looks good here transfers directly).
local PRESETS = {
    { name = "small",   size = 0.30, duration = 1.6 },
    { name = "medium",  size = 0.55, duration = 2.0 },
    { name = "large",   size = 0.95, duration = 2.4 },
    { name = "capital", size = 1.60, duration = 3.0 },
}

function ExplosionTestbed:onInit()
    -- Environment bootstrap (established SkyboxEntity pattern).
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

    -- Camera orbiting the centered explosion. The controller needs an
    -- anchor entity; without one it early-returns and never positions the
    -- camera (the "explosion behind the camera" bug). Anchor at origin,
    -- where the explosion lives, so it is always framed.
    local anchor = Registry:createEntity()
    Registry:add(anchor,
        require("Modules.Core.Components.NameComponent")("explosionAnchor"))
    -- Transform is REQUIRED: OrbitCameraController:updateCameraPosition
    -- early-returns without it (camera stuck at default pose).
    Registry:add(anchor, PhysicsComponents.Transform())
    local cam = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam)
    self.controllerOrbitCam = OrbitCameraController(cam, {
        distance = 4.0,
        minDistance = 0.5,
        maxDistance = 20.0,
        initialYaw = 0.0,
        initialPitch = 0.25,
    })
    self.controllerOrbitCam:setTarget(anchor)
    cam:get(CameraDataComponent):setController(self.controllerOrbitCam)
    CameraManager:setActiveCamera("OrbitCam")

    self.explosionSystem = ExplosionEffectSystem()
    self.entity       = nil  --- the single center explosion (or nil)
    self.elapsed      = 0.0
    self.spawnedTotal = 0
    self.serial       = 0
    self.seed         = rng:get31()
    self.presetIndex  = 2
    self.sizeScale     = 1.0
    self.durationScale = 1.0
    self.lastFrameMs   = 0.0
    self.frameAccum    = 0.0
    self.frameFrames   = 0
    self.avgFrameMs    = 0.0

    -- Start with the subject already on screen.
    self:spawnCenter(false)
end

---@private
---Despawn any live center explosion, then spawn a fresh one at the origin.
function ExplosionTestbed:spawnCenter(loop)
    self:despawnCenter()
    local preset = PRESETS[self.presetIndex]
    local serial = self.serial + 1
    self.serial = serial
    self.entity = ExplosionEffect(self.seed + serial * 7919, {
        position = Position(0, 0, 0),
        size = preset.size * self.sizeScale,
        duration = preset.duration * self.durationScale,
        loop = loop == true,
    })
    self.spawnedTotal = self.spawnedTotal + 1
end

---@private
function ExplosionTestbed:despawnCenter()
    if self.entity and self.entity:isValid() then
        Registry:destroyEntity(self.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    self.entity = nil
end

function ExplosionTestbed:onPreSim(data)
    local dt = data:deltaTime()
    for _, binding in pairs(Actions) do
        binding:update(dt)
    end

    -- F: restart the center explosion as a one-shot.
    if Actions.Spawn:isPressed() then
        self:spawnCenter(false)
    end

    -- K: toggle loop mode (one persistent cycling fireball).
    if Actions.LoopRing:isPressed() then
        local comp = self.entity and self.entity:isValid()
            and self.entity:get(ExplosionEffectComponent) or nil
        self:spawnCenter(not (comp and comp.loop))
    end

    -- Time scrubbing for detailed inspection: M freezes/unfreezes the
    -- effect clock; , / . step it back/forward while held (auto-freezes).
    if Actions.Freeze:isPressed() then
        self.timeFrozen = not self.timeFrozen
    end
    local scrub = 0.0
    if Actions.StepFwd:isDown() then
        scrub = scrub + 0.35
    end
    if Actions.StepBack:isDown() then
        scrub = scrub - 0.35
    end
    if scrub ~= 0.0 then
        self.timeFrozen = true
    end
    if self.timeFrozen then
        local comp = self.entity and self.entity:isValid()
            and self.entity:get(ExplosionEffectComponent) or nil
        if comp then
            comp.age = math.max(0.0, comp.age + scrub * dt)
        end
    end

    -- C: despawn (empty stage to compare against).
    if Actions.Clear:isPressed() then
        self:despawnCenter()
    end

    -- T: cycle scene-calibrated presets and respawn.
    if Actions.Preset:isPressed() then
        self.presetIndex = (self.presetIndex % #PRESETS) + 1
        self:spawnCenter(self:centerLoops())
    end

    -- Live parameter scaling: apply immediately to the running effect so
    -- the change is visible without a respawn.
    local resized = false
    if Actions.SizeUp:isPressed() then
        self.sizeScale = math.min(self.sizeScale + 0.25, 4.0)
        resized = true
    end
    if Actions.SizeDown:isPressed() then
        self.sizeScale = math.max(self.sizeScale - 0.25, 0.25)
        resized = true
    end
    local retimed = false
    if Actions.DurationUp:isPressed() then
        self.durationScale = math.min(self.durationScale + 0.25, 3.0)
        retimed = true
    end
    if Actions.DurationDown:isPressed() then
        self.durationScale = math.max(self.durationScale - 0.25, 0.5)
        retimed = true
    end

    local comp = self.entity and self.entity:isValid()
        and self.entity:get(ExplosionEffectComponent) or nil
    if comp then
        local preset = PRESETS[self.presetIndex]
        if resized then
            comp.size = preset.size * self.sizeScale
        end
        if retimed then
            comp.duration = preset.duration * self.durationScale
            if not comp.loop then
                comp.age = math.min(comp.age, comp.duration * 0.5)
            end
        end
    end
end

---@private
function ExplosionTestbed:centerLoops()
    local comp = self.entity and self.entity:isValid()
        and self.entity:get(ExplosionEffectComponent) or nil
    return (comp and comp.loop) == true
end

function ExplosionTestbed:onPostSim(data)
    -- Frozen = the effect clock stops entirely (system included), so
    -- scrubbed frames hold still for inspection.
    if not self.timeFrozen then
        ExplosionEffectSystem:update(data:deltaTime())
    end

    -- One-shot finished naturally -> clear the slot (loop entities persist).
    if self.entity and not self.entity:isValid() then
        self.entity = nil
    end
end

function ExplosionTestbed:onRender(data)
    RenderCoreSystem:render(data)

    -- Frame-cost bookkeeping for the HUD (30-frame window average).
    local dtMs = data:deltaTime() * 1000.0
    self.lastFrameMs = dtMs
    self.frameAccum = self.frameAccum + dtMs
    self.frameFrames = self.frameFrames + 1
    if self.frameFrames >= 30 then
        self.avgFrameMs = self.frameAccum / self.frameFrames
        self.frameAccum = 0.0
        self.frameFrames = 0
    end

    local lights = LightManager:getPointLights()
    local comp = self.entity and self.entity:isValid()
        and self.entity:get(ExplosionEffectComponent) or nil
    local preset = PRESETS[self.presetIndex]

    self:immediateUI(function()
        -- Lens flares draw in the backbuffer pass (house pattern: after
        -- the post chain, before HUD text).
        local flash = comp and math.exp(-comp.age * 2.6) or 0
        if comp and flash > 0.03 then
            local pos = self.entity:get(Physics.Transform):getPos()
            LensFlareSystem:drawAt(pos, {
                color = { 1.0, 0.72, 0.42 },
                intensity = 1.4 * flash,
                ghosts = 0.8,
                halo = 0.7,
            })
        end
        -- Sun stand-in: bright source IN FRONT of the camera (camera sits
        -- on +Z looking toward -Z/origin, so -Z is in frame).
        LensFlareSystem:drawAt(Position(0, 40, -300), {
            color = { 1.0, 0.95, 0.82 },
            intensity = 1.0,
            ghosts = 1.0,
            halo = 1.0,
        })
        local lines = {
            "Explosion Testbed",
            "F: restart   K: loop   C: empty stage",
            "T: preset   Q/E: size   Z/X: duration",
            "M: freeze   ,/.: scrub time (hold)",
            string.format("Preset %s   size %.2f   dur %.2fs   age %.2f%s",
                preset.name,
                comp and comp.size or 0,
                comp and comp.duration or 0,
                comp and comp.age or 0,
                self.timeFrozen and "  [FROZEN]" or ""),
            string.format("Frame: %.2f ms (avg %.2f ms)",
                self.lastFrameMs, self.avgFrameMs),
            "Shader: effect/explosion_volume.glsl (hot-reload on save)",
        }
        local y = 32
        for _, line in ipairs(lines) do
            DrawEx.TextAdditive("Unageo-Medium", line, 11, 32, y, 40, 20, 0.9, 0.9, 0.9, 0.95, 0.0, 0.5)
            y = y + 22
        end
    end)
end

return ExplosionTestbed
