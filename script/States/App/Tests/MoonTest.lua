local Application           = require('States.Application')

---@class MoonTest: Application
local MoonTest              = Subclass("MoonTest", Application)

local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local Materials             = require("Shared.Registries.Materials")
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
local CelestialComponents   = require("Modules.CelestialObjects.Components")

local RenderCoreSystem      = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraSystem          = require("Modules.Cameras.Systems.CameraSystem")

local SkyboxEntity          = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local MoonEntity            = require('Modules.CelestialObjects.Entities.MoonEntity')

---! still using legacy
local Primitive             = require("Legacy.Systems.Gen.Primitive")
local GenUtil               = require("Legacy.Systems.Gen.GenUtil")
local Generator             = require("Legacy.Systems.Gen.Generator")
local Starfield             = require("Legacy.Systems.Gen.Starfield")

function MoonTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.rng = RNG.FromTime()

    -- Timers
    self.timer = DeltaTimer("MoonTest")
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
    self.moonPos = Vec3f(0, 0, 0)
    self:createMoon(self.seed)

    -- Orbit camera targets station
    self.controllerOrbitCam:setTarget(self.moon)

    -- EventBus subscriptions
    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function MoonTest:createMoon(seed)
    local moonRNG = RNG.Create(seed)

    if self.moon then
        -- Remove old rigid bodies from physics world before destroying entities
        local oldMoonRb = self.moon:get(PhysicsComponents.RigidBody)
        if oldMoonRb and oldMoonRb:getRigidBody() then
            self.world:removeRigidBody(oldMoonRb:getRigidBody())
        end

        Registry:destroyEntity(self.moon, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

    local moonRNG = RNG.Create(seed + 12345)

    -- Moon
    local mesh = Primitive.IcoSphere(5)
    local meshAtmo = Primitive.IcoSphere(5, 1.5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    -- Example: choose body and slightly randomize colors
    local bodies = {
        { highland = Vec3f(0.72, 0.72, 0.72), maria = Vec3f(0.25, 0.25, 0.25) }, -- Moon
        { highland = Vec3f(0.74, 0.72, 0.68), maria = Vec3f(0.28, 0.27, 0.24) }, -- Mercury
        { highland = Vec3f(0.76, 0.74, 0.70), maria = Vec3f(0.30, 0.28, 0.25) }  -- Ceres
    }

    local body = moonRNG:choose(bodies)

    -- Slight randomization
    local function perturbColor(color, rng, amount)
        return Vec3f(
            Math.Clamp(color.x + rng:getUniformRange(-amount, amount), 0, 1),
            Math.Clamp(color.y + rng:getUniformRange(-amount, amount), 0, 1),
            Math.Clamp(color.z + rng:getUniformRange(-amount, amount), 0, 1)
        )
    end

    local moonOptions = {
        craterDensity     = 0.1,
        craterSharpness   = 0.47,
        mariaAmount       = 0.45,
        mountainHeight    = 1.0,
        mountainScale     = 1.0,
        proceduralBlend   = 0.85,
        brightRayStrength = 0.40,

        highlandColor     = perturbColor(body.highland, moonRNG, moonRNG:getUniformRange(0.002, 0.04)),
        mariaColor        = perturbColor(body.maria, moonRNG, moonRNG:getUniformRange(0.002, 0.06)),
        heightMult        = 0.045,
        enableAtmosphere  = false
    }

    print("HighlandColor:", moonOptions.highlandColor)
    print("MariaColor:", moonOptions.mariaColor)

    local texSurface = GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/moon', {
        seed              = moonRNG:getUniform(),
        craterDensity     = moonOptions.craterDensity,
        craterSharpness   = moonOptions.craterSharpness,
        mariaAmount       = moonOptions.mariaAmount,
        mountainHeight    = moonOptions.mountainHeight,
        mountainScale     = moonOptions.mountainScale,
        proceduralBlend   = moonOptions.proceduralBlend,
        rayCraterStrength = moonOptions.rayCraterStrength,
        brightRayStrength = moonOptions.brightRayStrength,
    })

    texSurface:genMipmap()
    texSurface:setMagFilter(TexFilter.Linear)
    texSurface:setMinFilter(TexFilter.LinearMipLinear)

    local matPlanet = Materials.MoonSurface()
    matPlanet:setTexture("surface", texSurface)

    self.moon = MoonEntity(seed, {
        { mesh = mesh, material = matPlanet },
    })
    local moonCmp = CelestialComponents.Gen.Moon(moonOptions)
    self.moon:add(moonCmp)
    local rbCmp = PhysicsComponents.RigidBody()
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rb:setKinematic(true)
    rb:setScale(10)
    rb:setPos(self.moonPos:toPosition())
    rbCmp:setRigidBody(rb)
    self.moon:add(rbCmp)
    -- Add moon rigid body to physics world
    self.world:addRigidBody(rb)

    -- Optionally set camera targets
    self.controllerOrbitCam:setTarget(self.moon)
end

---@param data EventData
function MoonTest:onStatePreRender(data)
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
function MoonTest:onRender(data)
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
function MoonTest:onInput(data)
    if Input:keyboard():isPressed(Button.KeyboardB) then
        self.seed = self.rng:get31()
        self:createMoon(self.seed)
        Log.Debug("New moon seed: %d", self.seed)
    end
end

---@param data EventData
function MoonTest:onStateSim(data)
    local dt = data:deltaTime()
    self.world:update(dt)
end

return MoonTest
