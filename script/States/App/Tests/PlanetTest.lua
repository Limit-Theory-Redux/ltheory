local Application         = require('States.Application')

---@class PlanetTest: Application
local PlanetTest          = Subclass("PlanetTest", Application)

local Registry            = require("Core.ECS.Registry")
local Materials           = require("Shared.Registries.Materials")
local CameraEntity        = require("Modules.Cameras.Entities").Camera
local PlanetEntity        = require('Modules.CelestialObjects.Entities.PlanetEntity')
local MoonEntity          = require('Modules.CelestialObjects.Entities.MoonEntity')
local AsteroidRingEntity  = require('Modules.CelestialObjects.Entities.AsteroidRingEntity')
local SkyboxEntity        = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local CoreComponents      = require('Modules.Core.Components')
local RenderComp          = require("Modules.Rendering.Components").Render
local CameraDataComponent = require('Modules.Cameras.Components.CameraDataComponent')
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local Entity              = require("Core.ECS.Entity")
local DrawEx              = require("UI.DrawEx")

-- New camera system
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")

---! still using legacy
local Primitive           = require("Legacy.Systems.Gen.Primitive")
local GenUtil             = require("Legacy.Systems.Gen.GenUtil")
local Material            = require("Legacy.GameObjects.Material")
local Generator           = require("Legacy.Systems.Gen.Generator")
local Starfield           = require("Legacy.Systems.Gen.Starfield")

function PlanetTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.ringRNG = RNG.FromTime()

    -- Timers
    self.timer = DeltaTimer("PlanetTest")
    self.timer:start("fps", 0.1)

    -- Double-click timer
    self.clickTimer = DeltaTimer("ClickTimer")
    self.clickCount = 0
    self.lastClickedBody = nil

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

    self.planetPos = Vec3f(0, 0, 0)

    -- Orbit camera state (initialize first)
    self.orbitAngle = 0.0
    self.orbitPitch = 0.0
    self.orbitRadius = 400.0
    self.targetOrbitRadius = 400.0

    -- NEW: Camera setup using CameraManager
    local cam = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam)

    -- Calculate initial camera position from orbit parameters
    local x = math.sin(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local y = math.sin(self.orbitPitch) * self.orbitRadius
    local z = math.cos(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local initialCamPos = Vec3f(self.planetPos.x + x, self.planetPos.y + y, self.planetPos.z + z)

    -- Set initial camera transform directly
    local transform = cam:get(PhysicsComponents.Transform)
    transform:setPos(Position(initialCamPos.x, initialCamPos.y, initialCamPos.z))

    local lookDir = (self.planetPos - initialCamPos):normalize()
    local rot = Quat.FromLook(lookDir, Vec3f(0, 1, 0))
    transform:setRot(rot)

    CameraManager:setActiveCamera("OrbitCam")
    self.orbitPitchMin = -0.8
    self.orbitPitchMax = 0.8
    self.zoomMinDistance = 1.0
    self.zoomSensitivity = 10
    self.zoomLerpSpeed = 2.0
    self.dragSensitivity = 0.005
    self.pitchSensitivity = 0.004
    self.autoRotationSpeed = 0.0225
    self.returnPitchLerp = 1.5
    self.isDragging = false
    self.dragReleaseTimer = 0
    self.dragReleaseDelay = 10.0

    self.focusEntity = nil
    self.enableRingDebug = true
    self.ringDebug = 1

    -- Camera transition state
    self.cameraTransition = {
        active = false,
        time = 0,
        duration = 1.5,
        startPos = Vec3f(0, 0, 0),
        targetPos = Vec3f(0, 0, 0),
        startAngle = 0,
        startPitch = 0,
        startRadius = 0,
        targetAngle = 0,
        targetPitch = 0,
        targetRadius = 0,
        targetEntity = nil
    }

    self:createPlanet(self.seed)

    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function PlanetTest:easeInOutCubic(t)
    if t < 0.5 then
        return 4 * t * t * t
    else
        local f = 2 * t - 2
        return 1 + f * f * f / 2
    end
end

function PlanetTest:createPlanet(seed)
    if self.planet then
        -- Remove old rigid bodies from physics world before destroying entities
        local oldPlanetRb = self.planet:get(PhysicsComponents.RigidBody)
        if oldPlanetRb and oldPlanetRb:getRigidBody() then
            self.world:removeRigidBody(oldPlanetRb:getRigidBody())
        end

        if self.ring then
            local oldRingRb = self.ring:get(PhysicsComponents.RigidBody)
            if oldRingRb and oldRingRb:getRigidBody() then
                self.world:removeRigidBody(oldRingRb:getRigidBody())
            end
        end

        if self.moons then
            for _, moon in ipairs(self.moons) do
                local moonRb = moon.entity:get(PhysicsComponents.RigidBody)
                if moonRb and moonRb:getRigidBody() then
                    self.world:removeRigidBody(moonRb:getRigidBody())
                end
            end
        end

        Registry:destroyEntity(self.planet, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

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
        { mesh = mesh,     material = self.matPlanet },
        { mesh = meshAtmo, material = self.matAtmo },
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

    if planetRNG:getUniform() < 0.2 then
        self:createPlanetRing(seed)
    else
        self.ringOuterRadius = self.planet:get(PhysicsComponents.RigidBody):getRadius()
    end

    self.planetRotationSpeed = planetRNG:getUniformRange(0.0005, 0.002)

    self:createMoons(seed)
    self:updateCameraZoomLimits()

    -- Set camera zoom to 4× planet radius
    local radius = rb:getScale()
    self.targetOrbitRadius = radius * 4
    self.orbitRadius = self.targetOrbitRadius

    -- Update camera to new position immediately
    self:updateOrbitCamera(0)
end

function PlanetTest:createPlanetRing(seed)
    local ringRNG = RNG.Create(seed)
    local planetRadius = self.planet:get(PhysicsComponents.RigidBody):getRadius()
    self.zoomMinDistance = planetRadius + 0.1

    local gap = planetRadius * 0.65
    local ringWidth = planetRadius * 2 * ringRNG:getExp()
    local innerRadius = planetRadius + gap
    local outerRadius = innerRadius + ringWidth
    self.ringOuterRadius = outerRadius

    local mesh = Primitive.Ring(innerRadius, outerRadius, 128)
    local ringTex = Tex2D.Create(512, 512, TexFormat.RGBA8)
    ringTex:clear(1, 1, 1, 1)

    self.matRing = Materials.PlanetRing()
    self.matRing:setTexture("ringTex", ringTex, Enums.UniformType.Tex2D)
    self.matRing:addStaticShaderVar("rMin", Enums.UniformType.Float, function() return innerRadius end)
    self.matRing:addStaticShaderVar("rMax", Enums.UniformType.Float, function() return outerRadius end)
    self.matRing:addStaticShaderVar("ringHeight", Enums.UniformType.Float, function() return 50 end)
    self.matRing:addStaticShaderVar("rotationSpeed", Enums.UniformType.Float, function() return 2.0 end)
    self.matRing:addStaticShaderVar("twistFactor", Enums.UniformType.Float, function() return 0.25 end)

    self.matRing:addStaticShaderVar("enableDebug", Enums.UniformType.Int, function() return self.enableRingDebug end)
    self.matRing:addStaticShaderVar("debugMode", Enums.UniformType.Int, function() return self.ringDebug end)

    self.ring = AsteroidRingEntity(seed, { { mesh = mesh, material = self.matRing } })

    local tiltDeg = ringRNG:getUniformRange(20, 35)
    local tiltDeg2 = ringRNG:getUniformRange(-20, -35)
    local tiltDeg3 = ringRNG:getUniformRange(80, 110)
    local tilt = ringRNG:choose({ tiltDeg, tiltDeg2, tiltDeg3 })
    local tiltRad = math.rad(tilt)

    local rbCmp = PhysicsComponents.RigidBody()
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z))
    rb:setRot(Quat.FromAxisAngle(Vec3f(1, 0, 0), tiltRad))
    rbCmp:setRigidBody(rb)
    self.ring:add(rbCmp)

    -- Add ring rigid body to physics world
    self.world:addRigidBody(rb)

    Registry:attachEntity(self.planet, self.ring)
end

function PlanetTest:createMoons(seed, numMoons)
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
        local minOrbit = self.ringOuterRadius + moonSize * 5
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

function PlanetTest:updateCameraZoomLimits()
    if not self.planet then return end

    local rb = self.planet:get(PhysicsComponents.RigidBody)
    local planetRadius = rb:getRadius()

    self.zoomMinDistance = planetRadius * 1.05
    self.zoomSensitivity = Math.Clamp(math.sqrt(planetRadius) * 0.02, 0.05, 10.0)

    -- Ensure current radius is valid
    self.targetOrbitRadius = math.max(self.zoomMinDistance, self.targetOrbitRadius)
end

function PlanetTest:updateCameraZoomLimitsForFocus()
    if not self.focusEntity then
        -- Reset to planet limits
        if self.planet then
            local rb = self.planet:get(PhysicsComponents.RigidBody)
            local planetRadius = rb:getRadius()
            self.zoomMinDistance = planetRadius * 1.05
            self.zoomSensitivity = Math.Clamp(math.sqrt(planetRadius) * 0.02, 0.05, 10.0)
        end
        return
    end

    local focusRb = self.focusEntity:get(PhysicsComponents.RigidBody)
    if focusRb and focusRb:getRigidBody() then
        local focusRadius = focusRb:getRadius()
        self.zoomMinDistance = focusRadius * 1.05
        self.zoomSensitivity = Math.Clamp(math.sqrt(focusRadius) * 0.02, 0.05, 10.0)
    end
end

-- NEW: Update orbit camera position based on angle, pitch, radius
function PlanetTest:updateOrbitCamera(dt)
    -- Get focus position
    local targetPos = self.planetPos
    if self.focusEntity then
        local focusRb = self.focusEntity:get(PhysicsComponents.RigidBody)
        if focusRb and focusRb:getRigidBody() then
            targetPos = focusRb:getRigidBody():getPos():toVec3f()
        else
            self.focusEntity = nil
            targetPos = self.planetPos
        end
    end

    -- Calculate camera position from orbit parameters
    local x = math.sin(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local y = math.sin(self.orbitPitch) * self.orbitRadius
    local z = math.cos(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local camPos = Vec3f(targetPos.x + x, targetPos.y + y, targetPos.z + z)

    -- Get the camera entity's transform component directly
    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end

    local transform = camEntity:get(PhysicsComponents.Transform)
    if not transform then return end

    -- Directly set transform position and rotation (bypass controller smoothing)
    transform:setPos(Position(camPos.x, camPos.y, camPos.z))

    -- Calculate look direction and set rotation
    local lookDir = (targetPos - camPos):normalize()
    local worldUp = Vec3f(0, 1, 0)
    local rot = Quat.FromLook(lookDir, worldUp)
    transform:setRot(rot)
end

function PlanetTest:onStatePreRender(data)
    local dt = data:deltaTime()
    local scaledDT = dt * (self.timeScale or 1)
    self.timer:update(dt)
    self.clickTimer:update(dt)

    if self.clickTimer:check("doubleClick") then
        self.clickCount = 0
        self.lastClickedBody = nil
    end

    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local fpsInterval = 0.1
        local instantFPS = self.frameCount / fpsInterval * (self.timeScale or 1)
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end

    if self.dragReleaseTimer > 0 then
        self.dragReleaseTimer = math.max(0, self.dragReleaseTimer - dt)
    end

    -- Handle camera transition animation
    if self.cameraTransition.active then
        self.cameraTransition.time = self.cameraTransition.time + dt
        local t = math.min(1.0, self.cameraTransition.time / self.cameraTransition.duration)
        local easedT = self:easeInOutCubic(t)

        -- Interpolate orbit parameters
        self.orbitAngle = self.cameraTransition.startAngle +
            (self.cameraTransition.targetAngle - self.cameraTransition.startAngle) * easedT
        self.orbitPitch = self.cameraTransition.startPitch +
            (self.cameraTransition.targetPitch - self.cameraTransition.startPitch) * easedT
        self.orbitRadius = self.cameraTransition.startRadius +
            (self.cameraTransition.targetRadius - self.cameraTransition.startRadius) * easedT
        self.targetOrbitRadius = self.cameraTransition.targetRadius

        if t >= 1.0 then
            self.cameraTransition.active = false
        end
    else
        -- Normal orbit behavior
        if not self.isDragging and self.dragReleaseTimer == 0 and (self.timeScale or 1) == 1 then
            self.orbitAngle = (self.orbitAngle or 0) + self.autoRotationSpeed * scaledDT
            self.orbitPitch = Math.Lerp(self.orbitPitch, 0.0, math.min(1, self.returnPitchLerp * dt))
        end

        -- Smooth zoom
        local lerpFactor = math.min(1, self.zoomLerpSpeed * dt)
        self.orbitRadius = self.orbitRadius + (self.targetOrbitRadius - self.orbitRadius) * lerpFactor
    end

    -- Update orbit camera
    self:updateOrbitCamera(dt)

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

    -- Update camera matrices
    CameraManager:updateViewMatrix()
    local resX, resY = Window:width(), Window:height()
    CameraManager:updateProjectionMatrix(resX, resY)
end

function PlanetTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local mem = GC.GetMemory()
        local camEntity = CameraManager:getActiveCameraEntity()
        local camTransform = camEntity and camEntity:get(PhysicsComponents.Transform)
        local camPos = camTransform and camTransform:getPos() or Position(0, 0, 0)

        local infoLines = {
            string.format("FPS: %d", math.floor(self.smoothFPS + 0.5)),
            string.format("Seed: %d", self.seed),
            string.format("Camera: (%.1f, %.1f, %.1f)", camPos.x, camPos.y, camPos.z),
            string.format("Orbit: Angle=%.1f° Pitch=%.1f° Radius=%.1f",
                math.deg(self.orbitAngle), math.deg(self.orbitPitch), self.orbitRadius),
            string.format("Freq: %.2f | Power: %.2f",
                self.genOptions.surfaceFreq, self.genOptions.surfacePower
            ),
            string.format("Ocean: %.2f | Clouds: %.2f",
                self.genOptions.oceanLevel, self.genOptions.cloudLevel
            ),
            string.format("Lua Memory: %.2f KB", mem),
            -- GC debug info
            string.format("GC Step Size: %d", GC.debug.stepSize),
            string.format("GC Last Mem After Cleanup: %.2f KB", GC.debug.lastMem or 0),
            string.format("GC Emergency: %s", GC.debug.emergencyTriggered and "YES" or "NO"),
            string.format("GC Spread Frames: %d", GC.debug.spreadFrames)
        }

        local y = 40
        for _, line in ipairs(infoLines) do
            DrawEx.TextAdditive('Unageo-Medium', line, 11,
                40, y, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
            y = y + 25
        end

        UI.DrawEx.TextAdditive(
            'Unageo-Medium',
            "Press [B] for new planet | Double-click to focus | Drag to rotate",
            14,
            self.resX / 2 - 14, self.resY - 60, 40, 20,
            1, 1, 1, 1,
            0.5, 0.5
        )
    end)
end

-- Mouse click handler for PlanetTest
function PlanetTest:handleMouseClick()
    local ray = CameraManager:mouseToRay(1e7)
    local result = self.world:rayCast(ray)

    if not result or not result.body then
        print("No hit - unfocusing")
        self.clickCount = 0
        self.lastClickedBody = nil
        self.focusEntity = nil
        self:updateCameraZoomLimitsForFocus()
        self.clickTimer:remove("doubleClick")
        return
    end

    local hitBody = result.body

    -- Check if this is the same object as last click
    if hitBody == self.lastClickedBody and self.clickTimer:has("doubleClick") then
        self.clickCount = self.clickCount + 1
    else
        self.clickCount = 1
        self.lastClickedBody = hitBody
        self.clickTimer:start("doubleClick", 0.3, false)
    end

    -- Double-click detected
    if self.clickCount == 2 then
        self:focusOnObject(result)
        self.clickCount = 0
        self.lastClickedBody = nil
        self.clickTimer:remove("doubleClick")
        return
    end

    -- Single click feedback
    if self.planet then
        local planetRb = self.planet:get(PhysicsComponents.RigidBody):getRigidBody()

        if hitBody == planetRb then
            print("CLICKED PLANET (click " .. self.clickCount .. "/2)")
        else
            -- Check moons
            local foundMoon = false
            if self.moons then
                for i, moon in ipairs(self.moons) do
                    local moonRb = moon.entity:get(PhysicsComponents.RigidBody):getRigidBody()
                    if hitBody == moonRb then
                        print("CLICKED MOON #" .. i .. " (click " .. self.clickCount .. "/2)")
                        foundMoon = true
                        break
                    end
                end
            end

            -- Check ring
            if not foundMoon and self.ring then
                local ringRb = self.ring:get(PhysicsComponents.RigidBody)
                if ringRb and hitBody == ringRb:getRigidBody() then
                    print("CLICKED RING (click " .. self.clickCount .. "/2)")
                end
            end
        end
    end
end

function PlanetTest:focusOnObject(raycastResult)
    local hitBody = raycastResult.body

    if not self.planet then return end

    local planetRb = self.planet:get(PhysicsComponents.RigidBody):getRigidBody()

    -- Focus on planet
    if hitBody == planetRb then
        print("=== FOCUSING ON PLANET ===")
        local planetRadius = planetRb:getBoundingRadius()
        local targetRadius = planetRadius * 3.0
        self:animateCameraToTarget(self.planet, planetRb:getPos(), targetRadius)
        return
    end

    -- Focus on moon
    if self.moons then
        for i, moon in ipairs(self.moons) do
            local moonRb = moon.entity:get(PhysicsComponents.RigidBody):getRigidBody()
            if hitBody == moonRb then
                print("=== FOCUSING ON MOON #" .. i .. " ===")
                local moonRadius = moonRb:getBoundingRadius()
                local targetRadius = moonRadius * 4.0
                self:animateCameraToTarget(moon.entity, moonRb:getPos(), targetRadius)
                return
            end
        end
    end

    -- Focus on ring
    if self.ring then
        local ringRb = self.ring:get(PhysicsComponents.RigidBody)
        if ringRb and hitBody == ringRb:getRigidBody() then
            print("=== FOCUSING ON RING ===")
            local planetRb = self.planet:get(PhysicsComponents.RigidBody):getRigidBody()
            local targetRadius = self.ringOuterRadius * 1.5
            self:animateCameraToTarget(self.ring, planetRb:getPos(), targetRadius, math.rad(45))
            return
        end
    end
end

function PlanetTest:animateCameraToTarget(targetEntity, targetPos, distance, forcePitch)
    -- Get current camera position
    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end

    local transform = camEntity:get(PhysicsComponents.Transform)
    if not transform then return end

    local currentCamPos = transform:getPos():toVec3f()
    local newTargetPos = targetPos:toVec3f()

    -- Calculate current angle/pitch/radius relative to the NEW target position
    -- This ensures we start from exactly where the camera currently is
    local dirFromNewTarget = currentCamPos - newTargetPos
    local currentRadius = dirFromNewTarget:length()

    if currentRadius < 0.01 then
        dirFromNewTarget = Vec3f(0, 0, 1)
        currentRadius = distance
    else
        dirFromNewTarget = dirFromNewTarget:normalize()
    end

    -- Calculate angle and pitch from current position relative to new target
    local currentAngle = math.atan2(dirFromNewTarget.x, dirFromNewTarget.z)
    local currentPitch = math.asin(Math.Clamp(dirFromNewTarget.y, -1, 1))

    -- Store starting state (relative to NEW target)
    self.cameraTransition.startAngle = currentAngle
    self.cameraTransition.startPitch = currentPitch
    self.cameraTransition.startRadius = currentRadius

    -- Calculate target angle/pitch
    -- Use a nice viewing angle for the target
    local targetAngle = currentAngle               -- Keep current angle by default
    local targetPitch = forcePitch or math.rad(30) -- Nice 30 degree viewing angle

    self.cameraTransition.targetAngle = targetAngle
    self.cameraTransition.targetPitch = targetPitch
    self.cameraTransition.targetRadius = distance

    -- Set the focus entity and update orbit params to match starting position
    self.focusEntity = targetEntity
    self.orbitAngle = currentAngle
    self.orbitPitch = currentPitch
    self.orbitRadius = currentRadius
    self.targetOrbitRadius = currentRadius -- Don't jump the target radius yet

    self:updateCameraZoomLimitsForFocus()

    -- Manually update camera position to ensure no jump
    self:updateOrbitCamera(0)

    -- Start transition
    self.cameraTransition.active = true
    self.cameraTransition.time = 0

    print("Camera transitioning to distance:", distance)
    print("Starting from angle:", math.deg(currentAngle), "pitch:", math.deg(currentPitch), "radius:", currentRadius)
    print("Target angle:", math.deg(targetAngle), "pitch:", math.deg(targetPitch), "radius:", distance)
end

function PlanetTest:onStateInput(data)
    local mouseState = Input:mouse()
    local scrollState = mouseState:scroll()

    if self.planet then
        local rb = self.planet:get(PhysicsComponents.RigidBody)
        local planetRadius = rb:getRadius()
        local planetPos = rb:getPos()

        local distance = math.max(1.0, self.orbitRadius)
        local surfaceDist = math.max(0.0, distance - planetRadius)
        local relativeDist = surfaceDist / planetRadius
        local zoomFactor = (relativeDist < 1.0) and (0.2 + 0.8 * relativeDist) or (1.0 + math.pow(relativeDist, 2.2))
        local baseSensitivity = (self.zoomSensitivity or 1.0) * math.sqrt(planetRadius) * 0.05

        -- Mouse scroll zoom
        local rawDelta = scrollState.y * baseSensitivity * zoomFactor
        if rawDelta > 0 then rawDelta = math.min(rawDelta, self.targetOrbitRadius - self.zoomMinDistance) end
        self.targetOrbitRadius = math.max(self.zoomMinDistance, self.targetOrbitRadius - rawDelta)

        -- Keyboard zoom with + and -
        local keyboardZoomDelta = 0
        if Input:isDown(Button.KeyboardEqual) or Input:isDown(Button.KeyboardNumpadAdd) then
            keyboardZoomDelta = baseSensitivity * zoomFactor * 2.0
        end
        if Input:isDown(Button.KeyboardMinus) or Input:isDown(Button.KeyboardNumpadSubtract) then
            keyboardZoomDelta = -baseSensitivity * zoomFactor * 2.0
        end

        if keyboardZoomDelta > 0 then
            keyboardZoomDelta = math.min(keyboardZoomDelta, self.targetOrbitRadius - self.zoomMinDistance)
        end
        self.targetOrbitRadius = math.max(self.zoomMinDistance, self.targetOrbitRadius - keyboardZoomDelta)
    end

    -- Right mouse drag for rotation
    if Input:isDown(Button.MouseRight) then
        local mouseDelta = mouseState:delta()
        if mouseDelta:length() > 0 then
            self.isDragging = true
            self.dragReleaseTimer = self.dragReleaseDelay
            self.orbitAngle = (self.orbitAngle or 0) + mouseDelta.x * self.dragSensitivity
            self.orbitPitch = Math.Clamp(
                (self.orbitPitch or 0) - mouseDelta.y * self.pitchSensitivity,
                self.orbitPitchMin, self.orbitPitchMax
            )
        end
    else
        if self.isDragging then self.dragReleaseTimer = self.dragReleaseDelay end
        self.isDragging = false
    end

    -- Generate new planet
    if Input:isPressed(Button.KeyboardB) then
        self.seed = self.ringRNG:get31()
        self:createPlanet(self.seed)
    end

    -- Mouse click for focusing
    if Input:isPressed(Button.MouseLeft) then
        self:handleMouseClick()
    end
end

---@param data EventData
function PlanetTest:onStateSim(data)
    self.world:update(data:deltaTime())
end

return PlanetTest
