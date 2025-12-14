local Application           = require('States.Application')

---@class ShipTest: Application
local ShipTest              = Subclass("ShipTest", Application)

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

local RenderCoreSystem      = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraSystem          = require("Modules.Cameras.Systems.CameraSystem")

local SkyboxEntity          = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local ShipGenerator         = require("Modules.Constructs.Managers.Generators.ShipGenerator")

---! still using legacy
local Generator             = require("Legacy.Systems.Gen.Generator")
local Starfield             = require("Legacy.Systems.Gen.Starfield")

function ShipTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.rng = RNG.FromTime()
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

    -- Create Ship using generator
    self.ShipPos = Vec3f(0, 0, 0)
    self:createShip(self.seed)

    -- Orbit camera targets Ship
    self.controllerOrbitCam:setTarget(self.Ship)

    -- EventBus subscriptions
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function ShipTest:createShip(seed)
    local ShipRNG = RNG.Create(seed)

    if self.Ship then
        -- Remove old rigid bodies from physics world before destroying entities
        local oldShipRb = self.Ship:get(PhysicsComponents.RigidBody)
        if oldShipRb and oldShipRb:getRigidBody() then
            self.world:removeRigidBody(oldShipRb:getRigidBody())
        end

        Registry:destroyEntity(self.Ship, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

    -- Use ShipGenerator to create a Ship
    self.Ship = ShipGenerator:createFighter(seed, {
        position = self.ShipPos:toPosition(),
        scale = 1.5,
        isKinematic = true
    })

    -- Add Ship's rigidbody to physics world
    local rbCmp = self.Ship:get(PhysicsComponents.RigidBody)
    local rb = rbCmp:getRigidBody()
    self.world:addRigidBody(rb)

    -- Optionally set camera targets
    self.controllerOrbitCam:setTarget(self.Ship)
end

---@param data EventData
function ShipTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local camPos = CameraManager:getActiveCameraEntity():get(CameraDataComponent):getController():getPosition()
        local infoLines = {
            string.format("FPS: %d", RenderCoreSystem:getSmoothFPS()),
            string.format("Frametime: %.2f ms", RenderCoreSystem:getSmoothFrameTime(true)),
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
function ShipTest:onInput(data)
    if Input:keyboard():isPressed(Button.KeyboardB) then
        self.seed = self.rng:get31()
        self:createShip(self.seed)
    end
end

---@param data EventData
function ShipTest:onStateSim(data)
    local dt = data:deltaTime()
    self.world:update(dt)
end

return ShipTest
