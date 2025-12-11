local Application                 = require('States.Application')

---@class CameraTest: Application
local CameraTest                  = Subclass("CameraTest", Application)

local Registry                    = require("Core.ECS.Registry")
local Entity                      = require("Core.ECS.Entity")
local Materials                   = require("Shared.Registries.Materials")
local DeltaTimer                  = require("Shared.Tools.DeltaTimer")
local DrawEx                      = require("UI.DrawEx")

local CameraEntity                = require("Modules.Cameras.Entities").Camera
local PlanetEntity                = require('Modules.CelestialObjects.Entities.PlanetEntity')
local MoonEntity                  = require('Modules.CelestialObjects.Entities.MoonEntity')
local AsteroidRingEntity          = require('Modules.CelestialObjects.Entities.AsteroidRingEntity')
local SkyboxEntity                = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local SpaceshipEntity             = require('Modules.Constructs.Entities.SpaceshipEntity')

local PhysicsComponents           = require("Modules.Physics.Components")
local CelestialComponents         = require("Modules.CelestialObjects.Components")
local CoreComponents              = require('Modules.Core.Components')
local RenderComp                  = require("Modules.Rendering.Components").Render
local CameraDataComponent         = require('Modules.Cameras.Components.CameraDataComponent')

local RenderCoreSystem            = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraSystem                = require("Modules.Cameras.Systems.CameraSystem")

local CameraManager               = require("Modules.Cameras.Managers.CameraManager")
local FreeCameraController        = require("Modules.Cameras.Managers.CameraControllers.FreeCameraController")
local OrbitCameraController       = require('Modules.Cameras.Managers.CameraControllers.OrbitCameraController')
local FirstPersonCameraController = require('Modules.Cameras.Managers.CameraControllers.FirstPersonCameraController')

---! still using legacy
local Primitive                   = require("Legacy.Systems.Gen.Primitive")
local GenUtil                     = require("Legacy.Systems.Gen.GenUtil")
local Generator                   = require("Legacy.Systems.Gen.Generator")
local Starfield                   = require("Legacy.Systems.Gen.Starfield")

function CameraTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.ringRNG = RNG.FromTime()

    -- Timers
    self.timer = DeltaTimer("CameraTest")
    self.timer:start("fps", 0.1)

    -- FPS tracking
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

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

    -- Camera setup with FreeCameraController
    local cam = CameraEntity()
    CameraManager:registerCamera("FreeCam", cam)
    local cam2 = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam2)
    local cam3 = CameraEntity()
    CameraManager:registerCamera("FirstPersonCam", cam3)

    -- Set controller
    self.controllerFreeCam = FreeCameraController(cam)
    cam:get(CameraDataComponent):setController(self.controllerFreeCam)

    self.orbitFreeCam = OrbitCameraController(cam2)
    self.orbitFreeCam:setTarget(nil) -- no target yet
    cam2:get(CameraDataComponent):setController(self.orbitFreeCam)

    self.firstPersonCam = FirstPersonCameraController(cam3)
    self.firstPersonCam:setTarget(nil) -- no target yet
    cam3:get(CameraDataComponent):setController(self.firstPersonCam)

    -- Activate free camera
    CameraManager:setActiveCamera("FreeCam")

    -- Set initial camera position
    self.planetPos = Vec3f(0, 0, 0)
    local initialPos = Position(0, 0, 500)
    self.controllerFreeCam:setPosition(initialPos)

    self.shipPos = Position(initialPos.x, initialPos.y, initialPos.z - 20)

    self.focusEntity = nil
    self.enableRingDebug = true
    self.ringDebug = 1

    self:createPlanet(self.seed)
    self:createShip(self.seed)

    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function CameraTest:createPlanet(seed)
    local planetRNG = RNG.Create(seed)

    local mesh      = Primitive.IcoSphere(5)
    local meshAtmo  = Primitive.IcoSphere(5, 1.5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    local genColor = function(rng)
        local h = rng:getUniformRange(0, 0.5)
        local l = Math.Saturate(rng:getUniformRange(0.2, 0.3) + 0.05 * rng:getExp())
        local s = rng:getUniformRange(0.1, 0.3)
        local c = Color.FromHSL(h, s, l)
        return Vec3f(c.r, c.g, c.b)
    end

    self.genOptions = {
        surfaceFreq  = 4 + planetRNG:getExp(),
        surfacePower = 1 + 0.5 * planetRNG:getExp(),
        surfaceCoef  = (planetRNG:getVec4(0.05, 1.00) ^ Vec4f(2, 2, 2, 2)):normalize(),
        color1       = genColor(planetRNG),
        color2       = genColor(planetRNG),
        color3       = genColor(planetRNG),
        color4       = genColor(planetRNG),
        oceanLevel   = planetRNG:getUniform() ^ 1.5,
        cloudLevel   = planetRNG:getUniformRange(-0.2, 0.15),
        atmoScale    = 1.1,
    }

    local texSurface = GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/planet', {
        seed = planetRNG:getUniform(),
        freq = self.genOptions.surfaceFreq,
        power = self.genOptions.surfacePower,
        coef = self.genOptions.surfaceCoef
    })

    self.matPlanet = Materials.PlanetSurface()
    self.matAtmo = Materials.PlanetAtmosphere()

    self.planet = PlanetEntity(seed, {
        { mesh = mesh, material = self.matPlanet },
        --{ mesh = meshAtmo, material = self.matAtmo },
    })

    local planetCmp = CelestialComponents.Gen.Planet(self.genOptions)
    self.planet:add(planetCmp)
    self.matPlanet:setTexture("surface", texSurface)

    local rbCmp = self.planet:get(PhysicsComponents.RigidBody)
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rbCmp:setRigidBody(rb)
    rb:setKinematic(true)
    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z))
    rb:setScale(planetRNG:getInt(100, 200))

    -- add rb to physics world
    self.world:addRigidBody(rb)

    self.planetRotationSpeed = planetRNG:getUniformRange(0.0005, 0.002)

    self:createMoons(seed)
end

function CameraTest:createMoons(seed, numMoons)
    if not self.planet then return end

    local moonRNG = RNG.Create(seed + 12345)
    numMoons = numMoons or moonRNG:getInt(1, 3)

    self.moons = self.moons or {}
    for _, moon in ipairs(self.moons) do
        Registry:destroyEntity(moon.entity)
    end
    self.moons = {}

    local planetRb = self.planet:get(PhysicsComponents.RigidBody)
    local planetRadius = planetRb:getRadius()
    local planetPos = planetRb:getPos()
    local baseSpeed = 0.00065

    for i = 1, numMoons do
        local moonSeed = seed + i * 1000
        local moonSize = planetRadius * (0.1 * moonRNG:getExp())
        local minOrbit = planetRadius + moonSize * 5
        local maxOrbit = (planetRadius + moonSize * 5) * 20
        local orbitRadius = moonRNG:getUniformRange(minOrbit, maxOrbit)
        local orbitSpeed = baseSpeed * math.sqrt(planetRadius / orbitRadius)
        local phase = moonRNG:getUniformRange(0, 2 * math.pi)
        local inclination = math.rad(moonRNG:getUniformRange(0, 180))

        -- Moon
        local mesh = Primitive.IcoSphere(4)
        local meshAtmo = Primitive.IcoSphere(4, 1.5)
        meshAtmo:computeNormals()
        meshAtmo:invert()

        local moonOptions = {
            craterDensity    = 4,
            craterSharpness  = 1,
            mariaAmount      = 0.3,
            highlandColor    = Vec3f(0.7, 0.68, 0.65),
            mariaColor       = Vec3f(0.25, 0.24, 0.23),
            heightMult       = 0.03,
            enableAtmosphere = true
        }

        local texSurface = GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/moon', {
            seed             = moonRNG:getUniform(),
            craterDensity    = moonOptions.craterDensity,
            craterSharpness  = moonOptions.craterSharpness,
            mariaAmount      = moonOptions.mariaAmount,
            highlandColor    = moonOptions.highlandColor,
            mariaColor       = moonOptions.mariaColor,
            heightMult       = moonOptions.heightMult,
            enableAtmosphere = moonOptions.enableAtmosphere
        })

        local matPlanet = Materials.MoonSurface()
        matPlanet:setTexture("surface", texSurface)

        local moon = MoonEntity(moonSeed, {
            { mesh = mesh, material = matPlanet },
        })

        local moonCmp = CelestialComponents.Gen.Moon(moonOptions)
        moon:add(moonCmp)

        local rbCmp = PhysicsComponents.RigidBody()
        local rb = RigidBody.CreateSphereFromMesh(mesh)
        rb:setKinematic(true)
        rb:setScale(moonSize)
        rb:setPos(Position(
            planetPos.x + math.cos(phase) * orbitRadius,
            planetPos.y + math.sin(inclination) * orbitRadius,
            planetPos.z + math.sin(phase) * orbitRadius
        ))
        rbCmp:setRigidBody(rb)
        moon:add(rbCmp)

        -- Add moon rigid body to physics world
        self.world:addRigidBody(rb)

        Registry:attachEntity(self.planet, moon)

        table.insert(self.moons, {
            entity = moon,
            radius = orbitRadius,
            speed = orbitSpeed,
            phase = phase,
            inclination = inclination
        })
    end
end

function CameraTest:createShip(seed)
    local shipRNG = RNG.Create(seed + 54321)

    local ShipGenerator = require("Modules.Constructs.Managers.ShipGenerator")

    self.ship = ShipGenerator:createFighter(seed, {
        position    = self.shipPos,
        scale       = 1,
        isKinematic = true,
    })

    local rbCmp = self.ship:get(PhysicsComponents.RigidBody)
    local rb = rbCmp:getRigidBody()
    rb:setScale(1.2)

    self.orbitFreeCam:setTarget(self.ship)
    self.firstPersonCam:setTarget(self.ship)

    -- Add ship rigid body to physics world
    self.world:addRigidBody(rb)
end

function CameraTest:onStatePreRender(data)
    local dt = data:deltaTime()
    local scaledDT = dt * (self.timeScale or 1)
    self.timer:update(dt)

    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local fpsInterval = 0.1
        local instantFPS = self.frameCount / fpsInterval * (self.timeScale or 1)
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end

    -- Update moon orbits
    if self.moons then
        for _, moon in ipairs(self.moons) do
            moon.phase = moon.phase + moon.speed * scaledDT
            local x = math.cos(moon.phase) * moon.radius
            local z = math.sin(moon.phase) * moon.radius
            local cosInc = math.cos(moon.inclination)
            local sinInc = math.sin(moon.inclination)
            local y = z * sinInc
            z = z * cosInc

            local newPos = Vec3f(
                self.planetPos.x + x,
                self.planetPos.y + y,
                self.planetPos.z + z
            )

            moon.entity:get(PhysicsComponents.RigidBody):getRigidBody():setPos(Position(newPos.x, newPos.y, newPos.z))
        end
    end

    -- Rotate planet
    if self.planet then
        local planetRb = self.planet:get(PhysicsComponents.RigidBody):getRigidBody()
        local currentRot = planetRb:getRot()
        local deltaRot = Quat.FromAxisAngle(Vec3f(0, 1, 0), self.planetRotationSpeed * scaledDT)
        planetRb:setRot(currentRot:mul(deltaRot))
    end

    -- FIX: Update orbit camera every frame to follow target
    --self.cameraControllers["orbit"]:onUpdate(dt)

    -- Update camera matrices
    CameraManager:updateViewMatrix()
    local resX, resY = Window:width(), Window:height()
    CameraManager:updateProjectionMatrix(resX, resY)
end

---@param data EventData
function CameraTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local mem = GC.GetMemory()
        local camPos = CameraManager:getActiveCameraEntity():get(CameraDataComponent):getController():getPosition()
        local yaw, pitch, roll = CameraManager:getActiveCameraEntity():get(CameraDataComponent):getController():getAngles()

        local infoLines = {
            string.format("FPS: %d", math.floor(self.smoothFPS + 0.5)),
            string.format("Seed: %d", self.seed),
            string.format("Camera: (%.1f, %.1f, %.1f)", camPos.x, camPos.y, camPos.z),
            string.format("Yaw: %.2f | Pitch: %.2f", math.deg(yaw), math.deg(pitch)),
            string.format("Lua Memory: %.2f KB", mem)
        }

        local y = 40
        for _, line in ipairs(infoLines) do
            DrawEx.TextAdditive('Unageo-Medium', line, 11,
                40, y, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
            y = y + 25
        end

        UI.DrawEx.TextAdditive(
            'Unageo-Medium',
            "WASD + Mouse to fly | Shift=Fast Alt=Slow",
            14,
            self.resX / 2 - 14, self.resY - 60, 40, 20,
            1, 1, 1, 1,
            0.5, 0.5
        )
    end)
end

---@param data EventData
function CameraTest:onInput(data)
    if Input:keyboard():isPressed(Button.KeyboardF1) then
        local currentCam = CameraManager:getActiveCameraName()
        if currentCam ~= "FreeCam" then
            CameraManager:setActiveCamera("FreeCam")
        end
    elseif Input:keyboard():isPressed(Button.KeyboardF2) then
        local currentCam = CameraManager:getActiveCameraName()
        if currentCam ~= "OrbitCam" then
            CameraManager:setActiveCamera("OrbitCam")
        end
    elseif Input:keyboard():isPressed(Button.KeyboardF3) then
        local currentCam = CameraManager:getActiveCameraName()
        if currentCam ~= "FirstPersonCam" then
            CameraManager:setActiveCamera("FirstPersonCam")
        end
    end
end

---@param data EventData
function CameraTest:onStateSim(data)
    self.world:update(data:deltaTime())
end

return CameraTest
