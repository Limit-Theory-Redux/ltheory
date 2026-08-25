local Application      = require('States.Application')

local Registry         = require("Core.ECS.Registry")
local CoreComponents   = require('Modules.Core.Components')
local PhysicsComponents = require("Modules.Physics.Components")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraEntity     = require("Modules.Cameras.Entities").Camera
local CameraDataComponent = require('Modules.Cameras.Components.CameraDataComponent')
local OrbitCameraController = require('Modules.Cameras.Managers.CameraControllers.OrbitCameraController')
local SkyboxEntity     = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraManager    = require("Modules.Cameras.Managers.CameraManager")
-- Keep-alive: CameraSystem drives controller input/pre-render updates.
local CameraSystem     = require("Modules.Cameras.Systems.CameraSystem")
local ExhaustPlume     = require("Modules.Constructs.Entities.ExhaustPlumeEntity")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructManager = require('Modules.Constructs.Managers.ConstructManager')
local HullEngineDiscovery = require('Modules.Constructs.Managers.Generators.HullEngineDiscovery')
local DrawEx           = require("UI.DrawEx")
local Actions          = require("Input.ActionBindings.ExplosionTestbedActions")
local ShipActions      = require("Input.ActionBindings.ShipActions")
local ShipFlightSystem = require("Modules.Constructs.Systems.ShipFlightSystem")

require('Shared.Definitions.MaterialDefs')
require('Shared.Definitions.UniformFuncDefs')

local rng = RNG.Create(0xE0C2) ---@diagnostic disable-line: lowercase-global

---@class ExhaustTestbed: Application
---Ship-focused sandbox for the volumetric exhaust plume effect: a real
---fighter (ConstructManager, same as the main game) with plumes attached
---at its engine nozzles, orbit camera on the ship. F toggles burn,
---T cycles plume-length presets.
---@field world Physics
---@field constructManager ConstructManager
---@field ship Entity
---@field shipHandle table
---@field hull { width: number, height: number, length: number, baseY: number }
---@field throttle number Current thrust level 0..1 (smooth)
---@field retroThrottle number Current retro (S) level 0..1 (smooth)
---@field retroPlumes Entity[] Nose retro thruster plumes
---@field shipEntity Entity|nil Ship handle root entity
---@field shipHandle table|nil ConstructManager createShip result
---@field plumes Entity[]
---@field burning boolean
---@field presetIndex integer
---@field lastFrameMs number|nil
local ExhaustTestbed = Subclass("ExhaustTestbed", Application)

---@type { name: string, lenFactor: number, radiusFactor: number, boost: number }[]
local LENGTH_PRESETS = {
    { name = "short",  lenFactor = 0.5, radiusFactor = 0.045, boost = 0.35 },
    { name = "cruise", lenFactor = 0.8, radiusFactor = 0.055, boost = 0.70 },
    { name = "boost",  lenFactor = 1.2, radiusFactor = 0.07,  boost = 1.00 },
}

function ExhaustTestbed:onInit()
    -- Full skybox world so the plumes get 3D context and depth cues.
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

    -- Real ship (production path), static. Game units: 1 unit = 1 m,
    -- so a ~50 m fighter gets scale 50.
    self.throttle = 0.0
    self.retroThrottle = 0.0
    self.rcsPlumes = {}
    self.rcsBoost = { right = 0, left = 0, up = 0, down = 0 }
    self.world = Physics.Create()
    self.constructManager = ConstructManager(Registry, self.world)
    self.shipScale = 50.0
    self.shipHandle = self.constructManager:createShip({
        seed = 4711, -- fixed: reproducible testbed scenario
        shipType = Enums.ShipType.Fighter,
        config = {
            position = Position(0, 0, 0),
            scale = self.shipScale,
            -- Frozen in place: the throttle drives the plume, not the
            -- ship's motion (testbed focus is the jet).
            isKinematic = true,
        },
    })
    self.ship = self.shipHandle.root

    -- Hull bounds (mesh-local units) for nozzle placement + framing.
    local shipData = self.ship:get(ConstructComponents.ShipData)
    local b = shipData:getGeneratedMesh():getBound()
    Log.Info("ExhaustTestbed: hull bound %s", tostring(b))
    self.hull = {
        width = b.upperx - b.lowerx,
        height = b.uppery - b.lowery,
        length = b.upperz - b.lowerz,
        baseY = b.uppery * 0.55,  -- engine row mid-upper hull
    }

    local cam = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam)
    self.controllerOrbitCam = OrbitCameraController(cam, {
        distance = self.hull.length * self.shipScale * 3.0,
        minDistance = self.hull.length * self.shipScale * 0.8,
        maxDistance = self.hull.length * self.shipScale * 20.0,
        initialYaw = 3.14, -- directly behind: the jets stretch straight away from the camera
        initialPitch = 0.55,
    })
    self.controllerOrbitCam:setTarget(self.ship)
    cam:get(CameraDataComponent):setController(self.controllerOrbitCam)
    CameraManager:setActiveCamera("OrbitCam")

    self.presetIndex = 2
    self.burning = true
    self.plumes = {}
    self:rebuildPlumes()
end

---@private
function ExhaustTestbed:clearPlumes()
    for _, plume in ipairs(self.plumes) do
        Registry:destroyEntity(plume, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    for _, plume in ipairs(self.retroPlumes or {}) do
        Registry:destroyEntity(plume, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    for _, list in pairs(self.rcsPlumes or {}) do
        for _, plume in ipairs(list) do
            Registry:destroyEntity(plume, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
    self.plumes = {}
    self.retroPlumes = {}
    self.rcsPlumes = {}
    self.plumeData = {}
end

---@private
---Plumes attached at the engine nozzles, placed from the ship's actual
---orientation (forward/up/right, from the rigid body) and connection
---points discovered on the generated hull (HullEngineDiscovery) - works
---on any procedural ship, not just this one.
function ExhaustTestbed:rebuildPlumes()
    self:clearPlumes()
    if not self.burning then
        return
    end
    local preset = LENGTH_PRESETS[self.presetIndex]
    local s = self.shipScale

    local rb = self.shipHandle.bodyComponent
    local rot = rb:getRot()
    local shipPos = self.ship:get(PhysicsComponents.Transform):getPos()

    -- Connection points on the hull (mesh-local), placed on the stern.
    local mesh = self.ship:get(ConstructComponents.ShipData):getGeneratedMesh()
    local engines = HullEngineDiscovery:discover(mesh, { count = 2 })
    if #engines == 0 then
        -- No stern faces found: fall back to the hull-center rear.
        engines = { {
            position = Vec3f(0, 0, -self.hull.length * 0.5),
            direction = Vec3f(0, 0, -1),
        } }
    end

    self.plumeData = {}
    for _, engine in ipairs(engines) do
        -- Mesh-local -> ship-local scale -> local offsets. Derived from
        -- the ship's transform each frame (syncPlumeTransforms) so the
        -- jets stay attached wherever the ship is.
        local lp = Vec3f(engine.position.x * s, engine.position.y * s, engine.position.z * s)
        local ld = Vec3f(engine.direction.x, engine.direction.y, engine.direction.z)
        local wp = rot:mulV(lp)
        local wd = rot:mulV(ld)
        local plume = ExhaustPlume(101 + #self.plumes, {
            position = Position(shipPos.x + wp.x, shipPos.y + wp.y, shipPos.z + wp.z),
            direction = { x = wd.x, y = wd.y, z = wd.z },
            length = self.hull.length * s * preset.lenFactor,
            radius = self.hull.height * s * preset.radiusFactor,
            boost = preset.boost,
        })
        self.plumes[#self.plumes + 1] = plume
        self.plumeData[#self.plumeData + 1] = { plume = plume, lp = lp }
    end

    -- Small forward-facing retro thrusters on the nose (speed reversal):
    -- S fires these; the jets stream along the ship's forward axis.
    -- Placed at the prong row (the hull's true front-mount faces, e.g.
    -- wing-tip/outrigger fronts) - centerline picks land in the recessed
    -- hull between prongs and the jets end up buried or floating.
    local retro = HullEngineDiscovery:discover(mesh, {
        count = 2,
        aft = Vec3f(0, 0, -1), -- nose faces -Z (forward axis)
        engineX = self.hull.width * 0.35,
    })
    for _, engine in ipairs(retro) do
        local lp = Vec3f(engine.position.x * s, engine.position.y * s, engine.position.z * s)
        local wd = rot:mulV(Vec3f(engine.direction.x, engine.direction.y, engine.direction.z))
        local wp = rot:mulV(lp)
        local plume = ExhaustPlume(201 + #self.retroPlumes, {
            position = Position(shipPos.x + wp.x, shipPos.y + wp.y, shipPos.z + wp.z),
            direction = { x = wd.x, y = wd.y, z = wd.z },
            length = self.hull.length * s * preset.lenFactor * 0.30,
            radius = self.hull.height * s * preset.radiusFactor * 0.45,
            boost = 0,
        })
        self.retroPlumes[#self.retroPlumes + 1] = plume
        self.plumeData[#self.plumeData + 1] = { plume = plume, lp = lp }
    end

    -- RCS-style lateral/vertical thrusters: pairs on the hull sides and
    -- top/bottom, tiny jets driven by A/D (right/left) and Space/Ctrl
    -- (up/down). Same discovery, different outward axes.
    local RCS = {
        { key = "right", aft = Vec3f(1, 0, 0),  xFactor = 0.5 },
        { key = "left",  aft = Vec3f(-1, 0, 0), xFactor = 0.5 },
        { key = "up",    aft = Vec3f(0, 1, 0),  xFactor = 0.3 },
        { key = "down",  aft = Vec3f(0, -1, 0), xFactor = 0.3 },
    }
    for _, rcs in ipairs(RCS) do
        local found = HullEngineDiscovery:discover(mesh, {
            count = 2,
            aft = rcs.aft,
            engineX = self.hull.width * rcs.xFactor,
            rearFraction = 0.5, -- rear-half band: typical RCS placement
        })
        local list = {}
        for _, engine in ipairs(found) do
            local lp = Vec3f(engine.position.x * s, engine.position.y * s, engine.position.z * s)
            local wd = rot:mulV(Vec3f(engine.direction.x, engine.direction.y, engine.direction.z))
            local wp = rot:mulV(lp)
            local plume = ExhaustPlume(301 + #self.retroPlumes + #list * 32, {
                position = Position(shipPos.x + wp.x, shipPos.y + wp.y, shipPos.z + wp.z),
                direction = { x = wd.x, y = wd.y, z = wd.z },
                length = self.hull.length * s * preset.lenFactor * 0.15,
                radius = self.hull.height * s * preset.radiusFactor * 0.35,
                boost = 0,
            })
            list[#list + 1] = plume
            self.plumeData[#self.plumeData + 1] = { plume = plume, lp = lp }
        end
        self.rcsPlumes[rcs.key] = list
    end
end

function ExhaustTestbed:onPreSim(data)
    local dt = data:deltaTime()
    for _, binding in pairs(ShipActions) do
        binding:update(dt)
    end
    for _, binding in pairs(Actions) do
        binding:update(dt)
    end

    -- W/S real thrust: W = rear burn (forward), S = nose retro burn
    -- (reverse). The two boost levels ramp smoothly and feed the plumes.
    local tz = ShipActions.ThrustZ:get()
    local rearTarget = math.max(0.0, tz)
    local retroTarget = math.max(0.0, -tz)
    self.throttle = self.throttle + (rearTarget - self.throttle) * math.min(1.0, dt * 6.0)
    self.retroThrottle = self.retroThrottle + (retroTarget - self.retroThrottle) * math.min(1.0, dt * 8.0)
    for _, plume in ipairs(self.plumes) do
        plume:setBoost(self.throttle)
    end
    for _, plume in ipairs(self.retroPlumes) do
        plume:setBoost(self.retroThrottle)
    end

    -- A/D (right/left) + Space/Ctrl (up/down): tiny RCS jets fire on the
    -- corresponding hull side/face.
    local tx = ShipActions.ThrustX:get()
    local ty = ShipActions.ThrustY:get()
    local rcsTargets = {
        right = math.max(0.0, tx),
        left = math.max(0.0, -tx),
        up = math.max(0.0, ty),
        down = math.max(0.0, -ty),
    }
    for key, target in pairs(rcsTargets) do
        local current = self.rcsBoost[key]
        self.rcsBoost[key] = current + (target - current) * math.min(1.0, dt * 8.0)
        for _, plume in ipairs(self.rcsPlumes[key] or {}) do
            plume:setBoost(self.rcsBoost[key])
        end
    end

    -- Plumes track the ship's transform (attached, wherever the ship is).
    local shipT = self.ship:get(PhysicsComponents.Transform)
    local shipPos = shipT:getPos()
    local rot = self.shipHandle.bodyComponent:getRot()
    for _, pd in ipairs(self.plumeData or {}) do
        local wp = rot:mulV(pd.lp)
        pd.plume:get(PhysicsComponents.Transform):setPos(
            Position(shipPos.x + wp.x, shipPos.y + wp.y, shipPos.z + wp.z))
    end

    if Actions.Spawn:isPressed() then
        self.burning = not self.burning
        self:rebuildPlumes()
    end
    if Actions.Preset:isPressed() then
        self.presetIndex = (self.presetIndex % #LENGTH_PRESETS) + 1
        self:rebuildPlumes()
    end
end

function ExhaustTestbed:onSim(data)
    local dt = data:deltaTime()
    self.world:update(dt)
end

function ExhaustTestbed:onRender(data)
    RenderCoreSystem:render(data)
    self.lastFrameMs = data:deltaTime() * 1000.0

    self:immediateUI(function()
        local preset = LENGTH_PRESETS[self.presetIndex]
        local lines = {
            "Exhaust Testbed (ship)",
            "W/S: thrust " .. string.format("%.0f%%", self.throttle * 100)
                .. "   T: preset (" .. preset.name .. ")",
            string.format("Plumes: %d   len %.1f   radius %.2f",
                #self.plumes,
                self.hull.length * self.shipScale * preset.lenFactor,
                self.hull.height * self.shipScale * preset.radiusFactor),
            string.format("Frame: %.2f ms", self.lastFrameMs or 0),
            "Shader: effect/exhaust_plume.glsl (hot-reload on save)",
        }
        local camEntity = CameraManager:getActiveCameraEntity()
        if camEntity then
            local camT = camEntity:get(PhysicsComponents.Transform)
            if camT then
                local cp = camT:getPos()
                lines[#lines + 1] = string.format(
                    "Cam: %.1f %.1f %.1f", cp.x, cp.y, cp.z)
            end
        end
        local y = 32
        for _, line in ipairs(lines) do
            DrawEx.TextAdditive("Unageo-Medium", line, 11, 32, y, 40, 20, 0.9, 0.9, 0.9, 0.95, 0.0, 0.5)
            y = y + 22
        end
    end)
end

function ExhaustTestbed:onCleanup()
    if self.world then
        self.world:destroy()
    end
end

return ExhaustTestbed
