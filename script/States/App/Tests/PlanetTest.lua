local Application         = require('States.Application')

---@class PlanetTest: Application
local PlanetTest          = Subclass("PlanetTest", Application)

local Registry            = require("Core.ECS.Registry")
local Materials           = require("Shared.Registries.Materials")
local CameraSystem        = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity        = require("Modules.Rendering.Entities").Camera
local PlanetEntity        = require('Modules.CelestialObjects.Entities.PlanetEntity')
local MoonEntity          = require('Modules.CelestialObjects.Entities.MoonEntity')
local AsteroidRingEntity  = require('Modules.CelestialObjects.Entities.AsteroidRingEntity')
local SkyboxEntity        = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local CoreComponents      = require('Modules.Core.Components')
local RenderComp          = require("Modules.Rendering.Components").Render
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local Entity              = require("Core.ECS.Entity")
local DrawEx              = require("UI.DrawEx")

---! still using legacy
local Primitive           = require("Legacy.Systems.Gen.Primitive")
local GenUtil             = require("Legacy.Systems.Gen.GenUtil")
local Material            = require("Legacy.GameObjects.Material")
local Generator           = require("Legacy.Systems.Gen.Generator")
local Starfield           = require("Legacy.Systems.Gen.Starfield")

function PlanetTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 3650
    self.ringRNG = RNG.FromTime()

    -- Timers
    self.timer = DeltaTimer("PlanetTest")
    self.timer:start("fps", 0.1)

    -- FPS tracking
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

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
    local cam = CameraEntity()
    CameraSystem:setCamera(cam)

    self.planetPos = Vec3f(0, 0, 0)

    -- Initialize camera transform
    self.camPos    = Vec3f(0, 0, -4)
    CameraSystem.currentCameraTransform:setPos(Position(self.camPos.x, self.camPos.y, self.camPos.z))
    CameraSystem.currentCameraTransform:setRot(Quat.LookAt(self.camPos, self.planetPos, Vec3f(0, 1, 0)))

    self.focusEntity       = nil

    self.targetCamRadius   = self.camPos.z
    self.zoomLerpSpeed     = 2.0
    self.zoomSensitivity   = 0.9
    self.zoomMinDistance   = 0.0
    self.isDragging        = false
    self.dragSensitivity   = 0.005
    self.dragReleaseTimer  = 0
    self.dragReleaseDelay  = 10.0
    self.angle             = 0.0
    self.pitch             = 0.0
    self.pitchMin          = -0.8
    self.pitchMax          = 0.8
    self.pitchSensitivity  = 0.004
    self.autoRotationSpeed = 0.0225
    self.returnPitchLerp   = 1.5

    self.enableRingDebug   = true
    self.ringDebug         = 1

    self:createPlanet(self.seed)

    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
end

function PlanetTest:createPlanet(seed)
    if self.planet then
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
    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z))
    rb:setScale(planetRNG:getInt(100, 200))

    self:createPlanetRing(seed)
    self:createMoons(seed)
    self:updateCameraZoomLimits()

    -- Set camera zoom to 4× planet radius
    local radius = rb:getScale()
    local planetPos = rb:getPos()
    local dir = (self.camPos - planetPos)
    if dir:length() == 0 then
        dir = Vec3f(0, 0, 1)
    else
        dir = dir:normalize()
    end
    self.targetCamRadius = radius * 4
    dir:imuls(self.targetCamRadius)
    planetPos:iadds(self.targetCamRadius)
    self.camPos = planetPos
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
        local inclination = math.rad(moonRNG:getUniformRange(0, 180)) -- tilt in degrees

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
    local planetPos = rb:getPos()

    self.zoomMinDistance = planetRadius * 1.05
    self.zoomSensitivity = Math.Clamp(math.sqrt(planetRadius) * 0.02, 0.05, 10.0)

    local safeDistance = planetRadius * 3.0
    local currentDist = (self.camPos - planetPos):length()

    if currentDist < self.zoomMinDistance then
        local dir = self.camPos - planetPos
        if dir:length() == 0 then
            dir = Vec3f(0, 0, 1)
        else
            dir = dir:normalize()
        end

        self.targetCamRadius = safeDistance
        self.zoomTransitionStart = self.camPos
        self.zoomTransitionEnd = planetPos + Vec3f(dir.x * safeDistance, dir.y * safeDistance, dir.z * safeDistance)
        self.zoomTransitionTime = 0
        self.zoomTransitionDuration = 1.0
    else
        self.targetCamRadius = math.max(self.zoomMinDistance, self.targetCamRadius)
    end
end

function PlanetTest:onStatePreRender(data)
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

    -- Smooth zoom
    local lerpFactor = math.min(1, self.zoomLerpSpeed * dt)
    self.camPos.z = self.camPos.z + (self.targetCamRadius - self.camPos.z) * lerpFactor

    if self.dragReleaseTimer > 0 then
        self.dragReleaseTimer = math.max(0, self.dragReleaseTimer - dt)
    end

    if not self.isDragging and self.dragReleaseTimer == 0 and (self.timeScale or 1) == 1 then
        self.angle = (self.angle or 0) + self.autoRotationSpeed * scaledDT
        self.pitch = Math.Lerp(self.pitch, 0.0, math.min(1, self.returnPitchLerp * dt))
    end

    -- Update moons orbit
    if self.moons then
        for _, moon in ipairs(self.moons) do
            moon.phase = moon.phase + moon.speed * scaledDT

            -- Orbit position in moon's local plane
            local x = math.cos(moon.phase) * moon.radius
            local z = math.sin(moon.phase) * moon.radius

            -- Apply inclination rotation around X-axis
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


    local radius = self.camPos.z
    local x = math.sin(self.angle) * math.cos(self.pitch) * radius
    local y = math.sin(self.pitch) * radius
    local z = math.cos(self.angle) * math.cos(self.pitch) * radius
    local camPos = Vec3f(x, y, z)
    local planetPos = self.planetPos

    local camTransform = CameraSystem.currentCameraTransform
    camTransform:setPos(Position(camPos.x, camPos.y, camPos.z))
    camTransform:setRot(Quat.LookAt(camPos, planetPos, Vec3f(0, 1, 0)))
end

function PlanetTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local mem = GC.GetMemory()
        local infoLines = {
            string.format("FPS: %d", math.floor(self.smoothFPS + 0.5)),
            string.format("Seed: %d", self.seed),
            string.format("Radius: %.2f | Zoom: %.2f",
                self.planet and self.planet:get(PhysicsComponents.RigidBody):getRadius() or 0,
                -self.camPos.z
            ),
            string.format("Freq: %.2f | Power: %.2f",
                self.genOptions.surfaceFreq, self.genOptions.surfacePower
            ),
            string.format("Ocean: %.2f | Clouds: %.2f",
                self.genOptions.oceanLevel, self.genOptions.cloudLevel
            ),
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
            "Press [B] for new planet",
            14,
            self.resX / 2 - 14, self.resY - 60, 40, 20,
            1, 1, 1, 1,
            0.5, 0.5
        )
    end)
end

-- Mouse click handler for PlanetTest
function PlanetTest:handleMouseClick(length)
    local mp = Input:mouse():position()
    local ray = CameraSystem:screenToRay(mp, length or 1e7)
    if not ray then return end

    print("raycast")

    -- self.focusEntity = closestEntity
end

function PlanetTest:onStateInput(data)
    local mouseState = Input:mouse()
    local scrollState = mouseState:scroll()

    self.camPos = self.camPos or Vec3f(0, 0, 50)
    self.targetCamRadius = self.targetCamRadius or self.camPos.z

    if self.planet then
        local rb = self.planet:get(PhysicsComponents.RigidBody)
        local planetRadius = rb:getRadius()
        local planetPos = rb:getPos()

        local distance = math.max(1.0, self.camPos:distance(planetPos))
        local surfaceDist = math.max(0.0, distance - planetRadius)
        local relativeDist = surfaceDist / planetRadius
        local zoomFactor = (relativeDist < 1.0) and (0.2 + 0.8 * relativeDist) or (1.0 + math.pow(relativeDist, 2.2))
        local baseSensitivity = (self.zoomSensitivity or 1.0) * math.sqrt(planetRadius) * 0.05

        -- Mouse scroll zoom
        local rawDelta = scrollState.y * baseSensitivity * zoomFactor
        if rawDelta > 0 then rawDelta = math.min(rawDelta, self.targetCamRadius - self.zoomMinDistance) end
        self.targetCamRadius = math.max(self.zoomMinDistance, self.targetCamRadius - rawDelta)

        -- Keyboard zoom with + and -
        local keyboardZoomDelta = 0
        if Input:isDown(Button.KeyboardEqual) or Input:isDown(Button.KeyboardNumpadAdd) then
            keyboardZoomDelta = baseSensitivity * zoomFactor * 2.0 -- Zoom in
        end
        if Input:isDown(Button.KeyboardMinus) or Input:isDown(Button.KeyboardNumpadSubtract) then
            keyboardZoomDelta = -baseSensitivity * zoomFactor * 2.0 -- Zoom out
        end

        if keyboardZoomDelta > 0 then
            keyboardZoomDelta = math.min(keyboardZoomDelta, self.targetCamRadius - self.zoomMinDistance)
        end
        self.targetCamRadius = math.max(self.zoomMinDistance, self.targetCamRadius - keyboardZoomDelta)
    end

    if Input:isDown(Button.MouseRight) then
        local mouseDelta = mouseState:delta()
        if mouseDelta:length() > 0 then
            self.isDragging = true
            self.dragReleaseTimer = self.dragReleaseDelay
            self.angle = (self.angle or 0) + mouseDelta.x * self.dragSensitivity
            self.pitch = Math.Clamp(
                (self.pitch or 0) - mouseDelta.y * self.pitchSensitivity,
                self.pitchMin, self.pitchMax
            )
        end
    else
        if self.isDragging then self.dragReleaseTimer = self.dragReleaseDelay end
        self.isDragging = false
    end

    if Input:isPressed(Button.KeyboardB) then
        self.seed = self.ringRNG:get31()
        self:createPlanet(self.seed)
    end

    if Input:isPressed(Button.MouseLeft) then
        self:handleMouseClick()
    end
end

return PlanetTest
