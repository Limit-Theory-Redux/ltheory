local Application         = require('States.Application')

---@class PlanetTest: Application
local PlanetTest          = Subclass("PlanetTest", Application)

local Registry            = require("Core.ECS.Registry")
local Materials           = require("Shared.Registries.Materials")
local CameraSystem        = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity        = require("Modules.Rendering.Entities").Camera
local PlanetEntity        = require('Modules.CelestialObjects.Entities.PlanetEntity')
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

    -- Timers
    self.timer = DeltaTimer("PlanetTest")
    self.timer:start("fps", 0.1)
    --self.timer:start("new_planet_tex", 120)

    -- FPS tracking
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

    -- Skybox
    ---@param entity Entity
    ---@param blendMode BlendMode
    self.skybox = SkyboxEntity(self.seed, function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)

        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
            ---@cast placeholder EmptyComponent
        end

        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local rng           = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle     = rng:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap  = Generator.Get('Nebula', rng)(rng, Config.gen.nebulaRes, placeholder.starDir)
            placeholder.irMap   = placeholder.envMap:genIRMap(256)
            placeholder.stars   = Starfield(rng, Config.gen.nStars(rng))
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
    CameraSystem.currentCameraTransform:setPosition(Position(self.camPos.x, self.camPos.y, self.camPos.z))
    CameraSystem.currentCameraTransform:setRotation(Quat.LookAt(self.camPos, self.planetPos, Vec3f(0, 1, 0)))

    self.targetCamRadius   = self.camPos.z -- target zoom distance
    self.zoomLerpSpeed     = 2.0           -- higher = faster
    self.zoomSensitivity   = 0.9           -- higher = faster
    self.zoomMinDistance   = 0.0           -- automatically set by planet radius
    self.isDragging        = false         -- is the user rotating manually
    self.dragStartAngle    = nil           -- store angle when drag starts
    self.dragSensitivity   = 0.005         -- adjust rotation speed
    self.dragReleaseTimer  = 0             -- delay before resuming auto-rotation
    self.dragReleaseDelay  = 1.0           -- seconds to wait after releasing right mouse
    self.angle             = 0.0           -- horizontal (yaw)
    self.pitch             = 0.0           -- vertical (pitch)
    self.pitchMin          = -0.8          -- ~-45 degrees
    self.pitchMax          = 0.8           -- ~+45 degrees
    self.pitchSensitivity  = 0.004         -- vertical rotation speed
    self.dragReleaseTimer  = 0
    self.dragReleaseDelay  = 1.0
    self.autoRotationSpeed = 0.0225
    self.returnPitchLerp   = 1.5 -- how fast to re-center pitch after release

    self:createPlanet(self.seed)

    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
end

function PlanetTest:createPlanet(seed)
    self.rng       = RNG.Create(seed)

    local mesh     = Primitive.IcoSphere(5)
    local meshAtmo = Primitive.IcoSphere(5, 1.5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    local genColor = function(rng)
        local h = rng:getUniformRange(0, 0.5)
        local l = Math.Saturate(rng:getUniformRange(0.2, 0.3) + 0.05 * rng:getExp())
        local s = rng:getUniformRange(0.1, 0.3)
        local c = Color.FromHSL(h, s, l)
        return Vec3f(c.r, c.g, c.b)
    end

    ---@type PlanetGenOptions
    self.genOptions = {
        surfaceFreq  = 4 + self.rng:getExp(),
        surfacePower = 1 + 0.5 * self.rng:getExp(),
        surfaceCoef  = (self.rng:getVec4(0.05, 1.00) ^ Vec4f(2, 2, 2, 2)):normalize(),
        color1       = genColor(self.rng),
        color2       = genColor(self.rng),
        color3       = genColor(self.rng),
        color4       = genColor(self.rng),
        oceanLevel   = self.rng:getUniform() ^ 1.5,
        cloudLevel   = self.rng:getUniformRange(-0.2, 0.15),
        atmoScale    = 1.1,
    }

    --todo: nonblocking?
    local texSurface = GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/planet', {
        seed = self.rng:getUniform(),
        freq = self.genOptions.surfaceFreq,
        power = self.genOptions.surfacePower,
        coef = self.genOptions.surfaceCoef
    })

    -- Clone materials
    self.matPlanet = Materials.PlanetSurface()
    self.matAtmo = Materials.PlanetAtmosphere()

    self.planet = PlanetEntity(seed, {
        { mesh = mesh,     material = self.matPlanet },
        { mesh = meshAtmo, material = self.matAtmo },
    })

    -- Attach gen data
    local planetCmp = CelestialComponents.Gen.Planet(self.genOptions)
    local genCmp = self.planet:add(planetCmp)

    self.matPlanet:setTexture("surface", texSurface)

    -- Physics
    local rbCmp = self.planet:get(PhysicsComponents.RigidBody)
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rbCmp:setRigidBody(rb)
    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z))

    self:createPlanetRing(seed)
end

function PlanetTest:createPlanetRing(seed)
    local rng = RNG.Create(seed)

    local planetRadius = self.planet:get(PhysicsComponents.RigidBody):getRadius()
    self.zoomMinDistance = planetRadius + 0.1

    local gap = planetRadius * 0.65
    local ringWidth = planetRadius * 2 * rng:getExp()

    local innerRadius = planetRadius + gap
    local outerRadius = innerRadius + ringWidth

    local mesh = Primitive.Ring(innerRadius, outerRadius, 128)
    mesh:getRadius()

    --todo: add tex2d generation
    local ringTex = Tex2D.Create(512, 512, TexFormat.RGBA8)
    ringTex:clear(1, 0, 1, 0.25)

    self.matRing = Materials.PlanetRing()
    --local tex = Tex2D.Create(256, 16, TexFormat.RGBA8)
    --tex:clear(1, 1, 1, 0.7) -- white with some transparency
    --self.matRing:setTexture("ringTex", tex)
    self.matRing:setTexture("ringTex", ringTex, Enums.UniformType.Tex2D)

    self.ring = AsteroidRingEntity(seed, { { mesh = mesh, material = self.matRing } })

    local ringGen = {
        innerRadius = innerRadius,
        outerRadius = outerRadius,
        angle = 0.0 -- will set tilt below
    }
    self.ring:add(CelestialComponents.Gen.PlanetRing(ringGen))

    --local transform = self.ring:get(CoreComponents.Transform)
    --transform:setRotation(Quat.FromAxisAngle(Vec3f(1, 0, 0), tiltRad))

    -- Update gen component with tilt
    --local genCmp = self.ring:get(CelestialComponents.Gen.PlanetRing)
    --genCmp.angle = tiltRad
    local tiltDeg = rng:getUniformRange(20, 35)
    local tiltDeg2 = rng:getUniformRange(-20, -35)
    local tiltDeg3 = rng:getUniformRange(80, 110)
    local tilt = rng:choose({ tiltDeg, tiltDeg2, tiltDeg3 })
    local tiltRad = math.rad(tilt)

    local rbCmp = PhysicsComponents.RigidBody()
    local rb = RigidBody.CreateSphere()                                       -- dummy, we control pos manually
    rb:setKinematic(true)
    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z)) -- same as planet
    rb:setRot(Quat.FromAxisAngle(Vec3f(1, 0, 0), tiltRad))
    rbCmp:setRigidBody(rb)
    self.ring:add(rbCmp)

    Registry:attachEntity(self.planet, self.ring)
end

function PlanetTest:onStatePreRender(data)
    local dt = data:deltaTime()
    local scaledDT = dt * (self.timeScale or 1)
    self.timer:update(dt)

    -- FPS calculation (scaled)
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

    -- Update drag release timer
    if self.dragReleaseTimer > 0 then
        self.dragReleaseTimer = math.max(0, self.dragReleaseTimer - dt)
    end

    -- Auto rotation (only yaw)
    if not self.isDragging and self.dragReleaseTimer == 0 and (self.timeScale or 1) == 1 then
        self.angle = (self.angle or 0) + self.autoRotationSpeed * scaledDT
    end

    -- Smoothly return pitch to 0 after releasing
    if not self.isDragging and self.dragReleaseTimer == 0 and (self.timeScale or 1) == 1 then
        self.pitch = Math.Lerp(self.pitch, 0.0, math.min(1, self.returnPitchLerp * dt))
    end

    -- Compute camera position from yaw/pitch
    local radius = self.camPos.z
    local x = math.sin(self.angle) * math.cos(self.pitch) * radius
    local y = math.sin(self.pitch) * radius
    local z = math.cos(self.angle) * math.cos(self.pitch) * radius

    local camPos = Vec3f(x, y, z)
    local planetPos = self.planetPos

    local camTransform = CameraSystem.currentCameraTransform
    camTransform:setPosition(Position(camPos.x, camPos.y, camPos.z))
    camTransform:setRotation(Quat.LookAt(camPos, planetPos, Vec3f(0, 1, 0)))
end

---@param data EventData
function PlanetTest:onRender(data)
    -- Normal rendering
    RenderCoreSystem:render(data)

    -- Immediate mode UI
    self:immediateUI(function()
        local mem = GC.GetMemory()
        DrawEx.TextAdditive('Unageo-Medium', self.fpsText, 20,
            40, 50, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        DrawEx.TextAdditive('Unageo-Medium', string.format("Lua Memory: %.2f KB", mem),
            20, 40, 70, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
    end)
end

---@param data EventData
function PlanetTest:onStateInput(data)
    local mouseState = Input:mouse()
    local scrollState = mouseState:scroll() -- Vec2f

    self.camPos = self.camPos or Vec3f(0, 0, 50)
    self.targetCamRadius = self.targetCamRadius or self.camPos.z

    -- Zoom target
    local scrollSensitivity = self.zoomSensitivity or 0.9
    local maxDeltaPerScroll = 10.0
    local delta = scrollState.y * scrollSensitivity
    delta = Math.Clamp(delta, -maxDeltaPerScroll, maxDeltaPerScroll)
    self.targetCamRadius = math.max(self.zoomMinDistance, self.targetCamRadius - delta)

    -- Manual rotation
    if Input:isDown(Button.MouseRight) then
        local mouseDelta = mouseState:delta() -- Vec2f
        if mouseDelta:length() > 0 then
            self.isDragging = true
            self.dragReleaseTimer = self.dragReleaseDelay

            -- Update yaw and pitch
            self.angle = (self.angle or 0) + mouseDelta.x * self.dragSensitivity
            self.pitch = Math.Clamp(
                (self.pitch or 0) - mouseDelta.y * self.pitchSensitivity,
                self.pitchMin, self.pitchMax
            )
        end
    else
        if self.isDragging then
            self.dragReleaseTimer = self.dragReleaseDelay
        end
        self.isDragging = false
    end
end

return PlanetTest
