--[[-- Controls ----------------------------------------------------------------

  There are 2 asteroids you can control. Hold Left Ctrl to control the first
  and Left Shift to control the second.

  Return      : Attach to/detach from the ship
  -/=         : Change the asteroid's scale
  I/J/K/L/U/O : Change the asteroid's position
  T/F/G/H/R/Y : Change the asteroid's rotation

  Scene:
  - An asteroid and a ship ahead of the player ship to run into, and a ship
    carrying two attached asteroids (compound bounding box).
  - A massive asteroid behind the player ship (small vs large collisions).
  - A trigger box near the origin and one attached to the player ship; their
    content counts are shown.

  Collider wireframes are drawn (GameState.debug.physics.drawWireframes).
----------------------------------------------------------------------------]]

local Application           = require('States.Application')
local Backdrop              = require("Render.Backdrop")

---@class PhysicsTest: Application
local PhysicsTest           = Subclass("PhysicsTest", Application)

local Entity                = require("Core.ECS.Entity")
local DrawEx                = require("UI.DrawEx")

local CameraEntity          = require("Modules.Cameras.Entities").Camera
local CameraManager         = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent   = require('Modules.Cameras.Components.CameraDataComponent')

local PhysicsComponents     = require("Modules.Physics.Components")
local CoreComponents        = require("Modules.Core.Components")
local RenderComp            = require("Modules.Rendering.Components").Render
local RenderCoreSystem      = require("Modules.Rendering.Systems.RenderCoreSystem")
local SkyboxEntity          = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local AsteroidMeshPool      = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local ShipGenerator         = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local Materials             = require("Shared.Registries.Materials")

local Generator             = require("Shared.Generation.Generator")
local Starfield             = require("Shared.Generation.Starfield")

local compoundTest          = true
local collisionTest         = true
local boundingTest          = true
local scaleTest             = true
local worldTriggerTest      = true
local attachedTriggerTest   = true

local ASTEROID_SEED         = 1234

function PhysicsTest:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 0
    self.world = Physics.Create()
    RenderCoreSystem:setDebugPhysics(self.world)
    GameState.debug.physics.drawWireframes = true

    self:createSkybox()

    local camOrbit = CameraEntity()
    CameraManager:registerCamera("OrbitCam", camOrbit)
    self.controllerOrbitCam = OrbitCameraController(camOrbit)
    camOrbit:get(CameraDataComponent):setController(self.controllerOrbitCam)
    CameraManager:setActiveCamera("OrbitCam")

    self:generate()
    self.controllerOrbitCam:setTarget(self.ship)

    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

--- The nebula backdrop (built once: it renders to textures, which cannot
--- happen inside an open pass, so it runs here with no blend mode).
function PhysicsTest:createSkybox()
    local skyboxFn = function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty) or entity:add(CoreComponents.Empty)
        if not placeholder.envMap then
            require("Shared.Generation.Nebula1")
            local nebulaRNG     = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle     = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap  = Generator.Get('Nebula', nebulaRNG)(nebulaRNG, Config.gen.nebulaRes, placeholder.starDir)
            placeholder.irMap   = placeholder.envMap:genIRMap(256)
            placeholder.stars   = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            CameraManager:setStarDir(placeholder.starDir)
            Renderer:setEnvironment(placeholder.envMap, placeholder.irMap)
        end
        Backdrop.draw(placeholder, blendMode)
    end
    self.skybox = SkyboxEntity(self.seed, skyboxFn)
    skyboxFn(self.skybox, nil)
end

--- An asteroid entity with a convex hull collider. `inWorld` false leaves its
--- body out of the physics world, for attaching it to another body.
---@return Entity entity
---@return RigidBody rb
function PhysicsTest:createAsteroid(scale, pos, inWorld)
    local mesh = AsteroidMeshPool:getFromSeed(ASTEROID_SEED):get(0)
    local entity = Entity.Create("AsteroidEntity",
        CoreComponents.Seed(ASTEROID_SEED),
        CoreComponents.Type("Asteroid"),
        PhysicsComponents.Transform())
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))

    local rb = RigidBody.CreateConvexHullFromMesh(mesh)
    rb:setScale(scale)
    rb:setPos(pos)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    if inWorld ~= false then self.world:addRigidBody(rb) end
    return entity, rb
end

---@return Entity entity
---@return RigidBody rb
function PhysicsTest:createShip(pos, rot)
    local ship = ShipGenerator:createFighter(self.seed, { position = pos, rotation = rot })
    local rb = ship:get(PhysicsComponents.RigidBody):getRigidBody()
    self.world:addRigidBody(rb)
    return ship, rb
end

function PhysicsTest:generate()
    self.ship, self.shipRb = self:createShip(Position(0, 0, 0))
    self.shipRb:setFriction(0)
    self.shipRb:setSleepThreshold(0, 0)

    if compoundTest then
        local _, rb1 = self:createAsteroid(5, Position(-10, 0, 10))
        local _, rb2 = self:createAsteroid(5, Position(10, 0, 10))
        -- The controlled asteroids, with the side of the ship they attach to.
        self.controlled = {
            { rb = rb1, offset = Vec3f(5, 0, 0) },
            { rb = rb2, offset = Vec3f(-5, 0, 0) },
        }
    end

    if collisionTest then
        self:createAsteroid(20, Position(20, 0, -100))
        self:createShip(Position(-20, 0, -100))

        local rot = Matrix.YawPitchRoll(0, 0, math.pi / 4):toQuat()
        local _, carrierRb = self:createShip(Position(0, 40, -100), rot)
        if boundingTest then
            local _, left = self:createAsteroid(5, Position(0, 0, 0), false)
            local _, right = self:createAsteroid(5, Position(0, 0, 0), false)
            carrierRb:attach(left, Vec3f(10, 0, 0), Quat.Identity())
            carrierRb:attach(right, Vec3f(-10, 0, 0), Quat.Identity())
        end
    end

    if scaleTest then
        self:createAsteroid(10000, Position(0, 0, 10500))
    end

    if worldTriggerTest then
        self.trigger1 = Trigger.CreateBox(Vec3f(20, 20, 20))
        self.trigger1:setPos(Position(9, 0, 0))
        self.world:addTrigger(self.trigger1)
    end

    if attachedTriggerTest then
        self.trigger2 = Trigger.CreateBox(Vec3f(20, 20, 20))
        self.world:addTrigger(self.trigger2)
        self.trigger2:attach(self.shipRb, Vec3f(0, 0, 0))
    end
end

--- Move, turn, scale and attach/detach the asteroids whose modifier is held.
function PhysicsTest:controlAsteroid(c)
    local rb = c.rb
    local attached = rb:getParentBody() ~= nil

    if Input:isPressed(Button.KeyboardEnter) then
        if attached then
            self.shipRb:detach(rb)
            self.world:addRigidBody(rb)
        else
            self.world:removeRigidBody(rb)
            self.shipRb:attach(rb, c.offset, Quat.Identity())
        end
        attached = not attached
    end

    if Input:isPressed(Button.KeyboardMinus) and rb:getScale() > 1 then rb:setScale(rb:getScale() - 1) end
    if Input:isPressed(Button.KeyboardEqual) then rb:setScale(rb:getScale() + 1) end

    local d = Position(0, 0, 0)
    if Input:isPressed(Button.KeyboardI) then d.z = d.z - 1 end
    if Input:isPressed(Button.KeyboardK) then d.z = d.z + 1 end
    if Input:isPressed(Button.KeyboardL) then d.x = d.x + 1 end
    if Input:isPressed(Button.KeyboardJ) then d.x = d.x - 1 end
    if Input:isPressed(Button.KeyboardO) then d.y = d.y + 1 end
    if Input:isPressed(Button.KeyboardU) then d.y = d.y - 1 end
    local pos = Position()
    if attached then
        rb:getPosLocal(pos)
        rb:setPosLocal(pos + d)
    else
        rb:getPos(pos)
        rb:setPos(pos + d)
    end

    local ypr = Vec3f(0, 0, 0)
    if Input:isPressed(Button.KeyboardT) then ypr.y = ypr.y - math.pi / 10 end
    if Input:isPressed(Button.KeyboardG) then ypr.y = ypr.y + math.pi / 10 end
    if Input:isPressed(Button.KeyboardH) then ypr.z = ypr.z - math.pi / 10 end
    if Input:isPressed(Button.KeyboardF) then ypr.z = ypr.z + math.pi / 10 end
    if Input:isPressed(Button.KeyboardY) then ypr.x = ypr.x - math.pi / 10 end
    if Input:isPressed(Button.KeyboardR) then ypr.x = ypr.x + math.pi / 10 end
    local turn = Matrix.YawPitchRoll(ypr.y, ypr.x, ypr.z):toQuat()
    local rot = Quat()
    if attached then
        rb:getRotLocal(rot)
        rb:setRotLocal(turn * rot)
    else
        rb:getRot(rot)
        rb:setRot(turn * rot)
    end
end

---@param data EventData
function PhysicsTest:onInput(data)
    if not self.controlled then return end
    if Input:isDown(Button.KeyboardControlLeft) then self:controlAsteroid(self.controlled[1]) end
    if Input:isDown(Button.KeyboardShiftLeft) then self:controlAsteroid(self.controlled[2]) end
end

---@param data EventData
function PhysicsTest:onStateSim(data)
    self.world:update(data:deltaTime())

    local collision = Collision()
    self.collisions = {}
    while self.world:getNextCollision(collision) do
        self.collisions[#self.collisions + 1] = string.format('Collision %d between %s and %s',
            collision.index, tostring(collision.body0), tostring(collision.body1))
    end
    self.collisionCount = collision.count
end

---@param data EventData
function PhysicsTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local lines = { string.format('Collision Count: %d', self.collisionCount or 0) }
        if worldTriggerTest then
            lines[#lines + 1] = string.format('World Trigger Count: %d', self.trigger1:getContentsCount())
        end
        if attachedTriggerTest then
            lines[#lines + 1] = string.format('Attached Trigger Count: %d', self.trigger2:getContentsCount())
        end
        for _, line in ipairs(self.collisions or {}) do lines[#lines + 1] = line end

        local y = 40
        for _, line in ipairs(lines) do
            DrawEx.TextAdditive('Unageo-Medium', line, 14, 40, y, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
            y = y + 25
        end
    end)
end

return PhysicsTest
