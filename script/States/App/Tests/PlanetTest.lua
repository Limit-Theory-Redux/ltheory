local Application       = require('States.Application')

---@class PlanetTest: Application
local PlanetTest        = Subclass("PlanetTest", Application)

local Registry          = require("Core.ECS.Registry")
local Materials         = require("Shared.Registries.Materials")
local CameraSystem      = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity      = require("Modules.Rendering.Entities").Camera
local PlanetEntity      = require('Modules.CelestialObjects.Entities.PlanetEntity')
local SkyboxEntity      = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents    = require('Modules.Core.Components')
local RenderComp        = require("Modules.Rendering.Components").Render
local RenderCoreSystem  = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer        = require("Shared.Tools.DeltaTimer")
local Entity            = require("Core.ECS.Entity")
local DrawEx            = require("UI.DrawEx")

---! still using legacy
local Primitive         = require("Legacy.Systems.Gen.Primitive")
local GenUtil           = require("Legacy.Systems.Gen.GenUtil")
local Material          = require("Legacy.GameObjects.Material")
local Generator         = require("Legacy.Systems.Gen.Generator")
local Starfield         = require("Legacy.Systems.Gen.Starfield")

function PlanetTest:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    -- Timers
    self.timer = DeltaTimer("PlanetTest")
    self.timer:start("fps", 0.1)

    -- FPS tracking
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

    -- Skybox
    ---@param entity Entity
    ---@param blendMode BlendMode
    self.skybox = SkyboxEntity(0, function(entity, blendMode)
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
    local camPos   = Vec3f(0, 0, -5)
    CameraSystem.currentCameraTransform:setPosition(Position(camPos.x, camPos.y, camPos.z))
    CameraSystem.currentCameraTransform:setRotation(Quat.LookAt(camPos, self.planetPos, Vec3f(0, 1, 0)))

    self:createPlanet(0)

    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

function PlanetTest:createPlanet(seed)
    local rng      = RNG.Create(seed)

    local mesh     = Primitive.IcoSphere(5)
    local meshAtmo = Primitive.IcoSphere(5, 1.5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    local genColor   = function(rng)
        local h = rng:getUniformRange(0, 0.5)
        local l = Math.Saturate(rng:getUniformRange(0.2, 0.3) + 0.05 * rng:getExp())
        local s = rng:getUniformRange(0.1, 0.3)
        local c = Color.FromHSL(h, s, l)
        return Vec3f(c.r, c.g, c.b)
    end

    -- store this where? "GenPlanetComponent"?
    local cloudLevel = rng:getUniformRange(-0.2, 0.15)
    local oceanLevel = rng:getUniform() ^ 1.5
    local atmoScale  = 1.1
    local color1     = genColor(rng)
    local color2     = genColor(rng)
    local color3     = genColor(rng)
    local color4     = genColor(rng)

    local texSurface = GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/planet', {
        seed = rng:getUniform(),
        freq = 4 + rng:getExp(),
        power = 1 + 0.5 * rng:getExp(),
        coef = (rng:getVec4(0.05, 1.00) ^ Vec4f(2, 2, 2, 2)):normalize()
    })

    --local matSurface = Material.Create("material/planet")
    --matSurface.state:setFloat('heightMult', 1.0)
    --matSurface.state:setFloat('oceanLevel', oceanLevel)
    --matSurface.state:setFloat3('color1', color1.x, color1.y, color1.z)
    --matSurface.state:setFloat3('color2', color2.x, color2.y, color2.z)
    --matSurface.state:setFloat3('color3', color3.x, color3.y, color3.z)
    --matSurface.state:setFloat3('color4', color4.x, color4.y, color4.z)
    --matSurface.state:setTexCube("surface", texSurface)
    --matSurface.state:setFloat3('starColor', 1.0, 0.5, 0.1)
    --matSurface.onUpdateState = function(shader, entity, eye)
    --    local body = entity:get(PhysicsComponents.RigidBody)
    --
    --    local origin = body:getPos():relativeTo(eye)
    --    shader:setFloat3('origin', origin.x, origin.y, origin.z)
    --    shader:setFloat('rPlanet', body:getScale())
    --    shader:setFloat('rAtmo', body:getScale() * atmoScale)
    --end
    --
    --local matAtmo = Material.Create("material/atmosphere")
    --matAtmo.blendMode = BlendMode.Alpha
    --matAtmo.state:setFloat3('starColor', 1.0, 0.5, 0.1)
    --matAtmo.onUpdateState = function(shader, entity, eye)
    --    local body = entity:get(PhysicsComponents.RigidBody)
    --
    --    local scale = body:getScale()
    --    shader:setFloat('rAtmo', scale * atmoScale)
    --    shader:setFloat('rPlanet', scale)
    --    local pos = body:getPos():relativeTo(eye)
    --    shader:setFloat3('origin', pos.x, pos.y, pos.z)
    --    shader:setFloat3('scale', scale, scale, scale)
    --end

    local planet     = PlanetEntity(seed,
        { { mesh = mesh, material = Materials.DebugColor }, { mesh = meshAtmo, material = Materials.DebugColor } })
    local rbCmp      = planet:get(PhysicsComponents.RigidBody)
    local rb         = RigidBody.CreateSphereFromMesh(mesh)
    rbCmp:setRigidBody(rb)

    rb:setPos(Position(self.planetPos.x, self.planetPos.y, self.planetPos.z))
end

function PlanetTest:onPreRender(data)
    local dt = data:deltaTime()
    self.timer:update(dt)
    self.time = self.time + dt

    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local instantFPS = self.frameCount * 10
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end

    local radius = 5.0
    local speed = 0.025
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
