local Application           = require('States.Application')

---@class StationTest: Application
local StationTest           = Subclass("StationTest", Application)

local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local DeltaTimer            = require("Shared.Tools.DeltaTimer")
local DrawEx                = require("UI.DrawEx")

local CameraEntity          = require("Modules.Cameras.Entities").Camera
local CameraManager         = require("Modules.Cameras.Managers.CameraManager")
local FreeCameraController  = require("Modules.Cameras.Managers.CameraControllers.FreeCameraController")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")

local PhysicsComponents     = require("Modules.Physics.Components")
local CoreComponents        = require("Modules.Core.Components")
local RenderComp            = require("Modules.Rendering.Components").Render
local CameraDataComponent   = require('Modules.Cameras.Components.CameraDataComponent')

local CameraSystem          = require("Modules.Cameras.Systems.CameraSystem")

local SkyboxEntity          = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local StationGenerator      = require("Modules.Constructs.Managers.StationGenerator")

---! still using legacy
local Generator             = require("Legacy.Systems.Gen.Generator")
local Starfield             = require("Legacy.Systems.Gen.Starfield")

function StationTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.rng = RNG.FromTime()

    -- Timers
    self.timer = DeltaTimer("StationTest")
    self.timer:start("fps", 0.1)
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"

    self.world = Physics.Create()

    -- Skybox
    self.skybox = SkyboxEntity(self.seed, function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)
        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
        end

        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local nebulaRNG     = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle     = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap  = Generator.Get('Nebula', nebulaRNG)(nebulaRNG, Config.gen.nebulaRes, placeholder.starDir)
            placeholder.irMap   = placeholder.envMap:genIRMap(256)
            placeholder.stars   = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            ShaderVar.PushFloat3('starDir', placeholder.starDir.x, placeholder.starDir.y, placeholder.starDir.z)
            ShaderVar.PushTexCube('envMap', placeholder.envMap)
            ShaderVar.PushTexCube('irMap', placeholder.irMap)
        end

        if blendMode == BlendMode.Disabled then
            RenderState.PushDepthWritable(false)
            local shader = Cache.Shader('farplane', 'skybox')
            RenderState.PushCullFace(CullFace.None)
            shader:start()
            Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
            shader:stop()
            RenderState.PopCullFace()
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

    -- Camera setup
    local camOrbit = CameraEntity()
    CameraManager:registerCamera("OrbitCam", camOrbit)

    self.controllerOrbitCam = OrbitCameraController(camOrbit)
    camOrbit:get(CameraDataComponent):setController(self.controllerOrbitCam)

    CameraManager:setActiveCamera("OrbitCam")

    -- Create station using generator
    self.stationPos = Vec3f(0, 0, 0)
    self:createStation(self.seed)

    -- Orbit camera targets station
    self.controllerOrbitCam:setTarget(self.station)

    -- EventBus subscriptions
    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function StationTest:createStation(seed)
    local stationRNG = RNG.Create(seed)

    if self.station then
        -- Remove old rigid bodies from physics world before destroying entities
        local oldStationRb = self.station:get(PhysicsComponents.RigidBody)
        if oldStationRb and oldStationRb:getRigidBody() then
            self.world:removeRigidBody(oldStationRb:getRigidBody())
        end

        Registry:destroyEntity(self.station, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

    -- Use StationGenerator to create a station
    self.station = StationGenerator:createStation(seed, {
        position = self.stationPos:toPosition(),
        scale = 1.5,
        isKinematic = true
    })

    -- Add station's rigidbody to physics world
    local rbCmp = self.station:get(PhysicsComponents.RigidBody)
    local rb = rbCmp:getRigidBody()
    self.world:addRigidBody(rb)

    -- Optionally set camera targets
    self.controllerOrbitCam:setTarget(self.station)
end

---@param data EventData
function StationTest:onStatePreRender(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local instantFPS = self.frameCount / 0.1
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end
end

---@param data EventData
function StationTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local camPos = CameraManager:getActiveCameraEntity():get(CameraDataComponent):getController():getPosition()
        local infoLines = {
            string.format("FPS: %d", math.floor(self.smoothFPS + 0.5)),
            string.format("Camera: (%.1f, %.1f, %.1f)", camPos.x, camPos.y, camPos.z),
            string.format("Seed: %d", self.seed)
        }

        local y = 40
        for _, line in ipairs(infoLines) do
            DrawEx.TextAdditive('Unageo-Medium', line, 11, 40, y, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
            y = y + 25
        end
    end)
end

---@param data EventData
function StationTest:onInput(data)
    if Input:keyboard():isPressed(Button.KeyboardB) then
        self.seed = self.rng:get31()
        self:createStation(self.seed)
    end
end

---@param data EventData
function StationTest:onStateSim(data)
    local dt = data:deltaTime()
    self.world:update(dt)
end

return StationTest
