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

    self.seed = 1000

    -- Timers
    self.timer = DeltaTimer("PlanetTest")
    self.timer:start("fps", 0.1)
    self.timer:start("new_planet_tex", 10)

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
    self.camPos    = Vec3f(0, 0, -2)
    CameraSystem.currentCameraTransform:setPosition(Position(self.camPos.x, self.camPos.y, self.camPos.z))
    CameraSystem.currentCameraTransform:setRotation(Quat.LookAt(self.camPos, self.planetPos, Vec3f(0, 1, 0)))

    self:createPlanet(self.seed)

    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
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
    local texSurface = GenUtil.ShaderToTexCube(1024, TexFormat.RGBA16F, 'gen/planet', {
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

    --self:createPlanetRing(seed)
end

function PlanetTest:createPlanetRing(seed)
    local rng = RNG.Create(seed)

    local innerRadius = 1.6 + 0.4 * rng:getExp()
    local outerRadius = innerRadius + 0.8 + 0.6 * rng:getExp()
    local mesh = Primitive.Ring(innerRadius, outerRadius, 128)

    local ringTex = GenUtil.ShaderToTexCube(1024, TexFormat.RGBA8, "gen/planetring", {
        seed = rng:getUniform(),
        freq = 0.8 + 0.6 * rng:getExp(),
        power = 1.2 + 0.8 * rng:getExp(),
        innerRadius = 0.35 + 0.15 * rng:getUniform(),
        outerRadius = 0.85 + 0.10 * rng:getUniform(),
        ringDensity = 0.6 + 0.4 * rng:getExp(),
        dustScale = 1.5 + 1.5 * rng:getExp(),
    })

    self.matRing = Materials.PlanetRing()

    self.ring = AsteroidRingEntity(seed, { { mesh = mesh, material = self.matRing } })

    self.matRing:setTexture("ringTex", ringTex)

    local rbCmp = ring:get(PhysicsComponents.RigidBody)
    local rb = RigidBody.CreateSphere() -- temp sphere
    rbCmp:setRigidBody(rb)
    rb:setKinematic(true)

    local ringGen = {
        innerRadius = innerRadius,
        outerRadius = outerRadius,
        seed = rng:getUniform(),
    }
    self.ring:add(CelestialComponents.Gen.Ring(ringGen))
end

function PlanetTest:onPreRender(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local instantFPS = self.frameCount * 10
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end

    if self.timer:check("new_planet_tex") then
        local texSurface = GenUtil.ShaderToTexCube(1024, TexFormat.RGBA16F, 'gen/planet', {
            seed = self.rng:getUniform(),
            freq = self.genOptions.surfaceFreq,
            power = self.genOptions.surfacePower,
            coef = self.genOptions.surfaceCoef
        })

        self.matPlanet:setTexture("surface", texSurface)
    end

    local radius = self.camPos.z
    local speed = 0.04
    local angle = (self.angle or 0) + speed * dt
    self.angle = angle

    local x = math.sin(angle) * radius
    local z = math.cos(angle) * radius
    local y = 0.0

    local camPos = Vec3f(x, y, z)
    local planetPos = self.planetPos

    local camTransform = CameraSystem.currentCameraTransform
    camTransform:setPosition(Position(camPos.x, camPos.y, camPos.z))
    camTransform:setRotation(Quat.LookAt(camPos, planetPos, Vec3f(0, 1, 0)))
end

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

return PlanetTest
