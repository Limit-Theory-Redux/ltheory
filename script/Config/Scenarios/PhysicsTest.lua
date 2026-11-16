-- Physics Test: the physics test scene next to the player's ship, with the
-- colliders drawn as wireframes and the collisions and trigger contents on
-- screen. (Was the PhysicsTest state.)
--
-- Around the ship (in the ship's size, ahead = the way it faces):
-- - two asteroids beside it that can be moved, turned, scaled and attached to
--   the ship (compound body);
-- - ahead: an asteroid, a ship, and a carrier ship rolled 45 degrees with two
--   asteroids attached (compound collider and bounds);
-- - far ahead: a huge asteroid (a collider at a very different scale);
-- - a trigger box beside the ship, and one attached to it.
--
-- Keys (the numpad; the test's own keys are flight keys in the game):
-- / and * select asteroid 1 or 2 (again: none); with one selected, Enter
-- attaches or detaches it, + and - scale it, 8 2 4 6 9 3 move it (ahead,
-- back, left, right, up, down) and, with 5 held, turn it.

local Entity = require("Core.ECS.Entity")
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderComp = require("Modules.Rendering.Components").Render
local AsteroidMeshPool = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local Materials = require("Shared.Registries.Materials")
local ShipGenerator = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local Overlay = require("Config.Scenarios.Overlay")

local ASTEROID_SEED = 1
local TURN_STEP = math.pi / 10

-- Test units: the test was built around a ship of size 1; here 1 = the
-- player's ship (Config.game.shipHulls.scale[1])
local function unit()
    return Config.game.shipHulls.scale[1]
end

local function shipBody(game)
    return game.playerShip:get(PhysicsComponents.RigidBody):getRigidBody()
end

---A position `x, y, z` test units from the player's ship.
local function at(game, x, y, z)
    local p = shipBody(game):getPos()
    local u = unit()
    return Position(p.x + x * u, p.y + y * u, p.z + z * u)
end

---An asteroid with a convex hull collider, in the world unless `attached`.
local function createAsteroid(game, scale, pos, attached)
    local mesh = AsteroidMeshPool:getFromSeed(ASTEROID_SEED):get(0)
    local entity = Entity.Create("AsteroidEntity",
        CoreComponents.Seed(ASTEROID_SEED),
        CoreComponents.Type("Asteroid"),
        PhysicsComponents.Transform())
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))

    local rb = RigidBody.CreateConvexHullFromMesh(mesh)
    rb:setScale(scale * unit())
    rb:setPos(pos)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    game:addSceneEntity(entity, { addBody = not attached })
    return entity, rb
end

local function createShip(game, pos, rot)
    local ship = ShipGenerator:createFighter(game.rng:get31(), { position = pos, rotation = rot, scale = unit() })
    game:addSceneEntity(ship, { addBody = true })
    return ship:get(PhysicsComponents.RigidBody):getRigidBody()
end

local function setup(game)
    local data = game.scenarioData
    local u = unit()

    data.wireframes = GameState.debug.physics.drawWireframes
    GameState.debug.physics.drawWireframes = true

    -- Beside the ship: the two asteroids that can be controlled, with the side
    -- of the ship they attach to
    local _, rb1 = createAsteroid(game, 5, at(game, -10, 0, 10))
    local _, rb2 = createAsteroid(game, 5, at(game, 10, 0, 10))
    data.controlled = {
        { rb = rb1, offset = Vec3f(5 * u, 0, 0) },
        { rb = rb2, offset = Vec3f(-5 * u, 0, 0) },
    }

    -- Ahead: collisions, and a compound collider
    createAsteroid(game, 20, at(game, 20, 0, 100))
    createShip(game, at(game, -20, 0, 100))
    local carrierRb = createShip(game, at(game, 0, 40, 100), Matrix.YawPitchRoll(0, 0, math.pi / 4):toQuat())
    local _, left = createAsteroid(game, 5, Position(0, 0, 0), true)
    local _, right = createAsteroid(game, 5, Position(0, 0, 0), true)
    carrierRb:attach(left, Vec3f(10 * u, 0, 0), Quat.Identity())
    carrierRb:attach(right, Vec3f(-10 * u, 0, 0), Quat.Identity())
    data.carrier = { rb = carrierRb, children = { left, right } }

    -- Far ahead: a collider at a very different scale
    createAsteroid(game, 10000, at(game, 0, 0, 10500))

    -- Triggers: one in the world beside the ship, one attached to it
    data.worldTrigger = Trigger.CreateBox(Vec3f(20 * u, 20 * u, 20 * u))
    data.worldTrigger:setPos(at(game, 9, 0, 0))
    game.world:addTrigger(data.worldTrigger)
    data.shipTrigger = Trigger.CreateBox(Vec3f(20 * u, 20 * u, 20 * u))
    game.world:addTrigger(data.shipTrigger)
    data.shipTrigger:attach(shipBody(game), Vec3f(0, 0, 0))

    data.collisions = {}
    data.collisionCount = 0

    -- Orbiting the ship from far enough out to see the scene around it
    game:setOrbitFocus(game.playerShip, { distance = 60 * u, minDistance = 2 * u, maxDistance = 30000 * u, zoomRatio = 0.15 })
end

---Move, turn, scale and attach/detach the selected asteroid.
local function controlAsteroid(game, c)
    local rb = c.rb
    local ship = shipBody(game)
    local u = unit()
    local attached = rb:getParentBody() ~= nil

    if Input:isPressed(Button.KeyboardNumpadEnter) then
        if attached then
            ship:detach(rb)
            game.world:addRigidBody(rb)
        else
            game.world:removeRigidBody(rb)
            ship:attach(rb, c.offset, Quat.Identity())
        end
        attached = not attached
    end

    if Input:isPressed(Button.KeyboardNumpadSubtract) and rb:getScale() > 1.5 * u then rb:setScale(rb:getScale() - u) end
    if Input:isPressed(Button.KeyboardNumpadAdd) then rb:setScale(rb:getScale() + u) end

    -- 8 2 4 6 9 3: ahead, back, left, right, up, down; with 5 held they turn
    local x, y, z = 0, 0, 0
    if Input:isPressed(Button.KeyboardNumpad8) then z = z + 1 end
    if Input:isPressed(Button.KeyboardNumpad2) then z = z - 1 end
    if Input:isPressed(Button.KeyboardNumpad6) then x = x + 1 end
    if Input:isPressed(Button.KeyboardNumpad4) then x = x - 1 end
    if Input:isPressed(Button.KeyboardNumpad9) then y = y + 1 end
    if Input:isPressed(Button.KeyboardNumpad3) then y = y - 1 end
    if x == 0 and y == 0 and z == 0 then return end

    if Input:isDown(Button.KeyboardNumpad5) then
        -- pitch with 8/2, yaw with 4/6, roll with 9/3
        local turn = Matrix.YawPitchRoll(x * TURN_STEP, z * TURN_STEP, y * TURN_STEP):toQuat()
        local rot = Quat()
        if attached then
            rb:getRotLocal(rot)
            rb:setRotLocal(turn * rot)
        else
            rb:getRot(rot)
            rb:setRot(turn * rot)
        end
    else
        local d = Position(x * u, y * u, z * u)
        local pos = Position()
        if attached then
            rb:getPosLocal(pos)
            rb:setPosLocal(pos + d)
        else
            rb:getPos(pos)
            rb:setPos(pos + d)
        end
    end
end

return {
    name = "Physics Test",
    description = "Colliders, compound bodies and triggers next to your ship, drawn as wireframes.",
    stations = false,

    -- Above the planet's ring, clear of its rocks
    spawnShip = function(game)
        local p = game:defaultShipSpawn()
        return Position(p.x, p.y + 30, p.z)
    end,

    setup = setup,

    onInput = function(game, dt)
        local data = game.scenarioData
        if Input:isPressed(Button.KeyboardNumpadDivide) then
            data.selected = data.selected ~= 1 and 1 or nil
        elseif Input:isPressed(Button.KeyboardNumpadMultiply) then
            data.selected = data.selected ~= 2 and 2 or nil
        end
        if data.selected then
            controlAsteroid(game, data.controlled[data.selected])
        end
    end,

    -- The collisions of this step (after the physics step)
    postUpdate = function(game, dt)
        local data = game.scenarioData
        local collision = Collision()
        data.collisions = {}
        while game.world:getNextCollision(collision) do
            if #data.collisions < 8 then
                data.collisions[#data.collisions + 1] = string.format("Collision %d between %s and %s",
                    collision.index, tostring(collision.body0), tostring(collision.body1))
            end
        end
        data.collisionCount = collision.count
    end,

    drawOverlay = function(game)
        local data = game.scenarioData
        local lines = {
            string.format("Physics Test: %d collisions    world trigger: %d    ship trigger: %d",
                data.collisionCount or 0, data.worldTrigger:getContentsCount(), data.shipTrigger:getContentsCount()),
            "Numpad: / * select asteroid (" .. (data.selected and tostring(data.selected) or "none") ..
                ")    Enter attach    + - scale    8 2 4 6 9 3 move    hold 5 to turn",
        }
        for _, line in ipairs(data.collisions or {}) do lines[#lines + 1] = line end
        Overlay.draw(lines)
    end,

    -- Attachments and triggers go before the bodies do (the scene teardown
    -- removes the entities)
    teardown = function(game)
        local data = game.scenarioData
        local ship = shipBody(game)
        for _, c in ipairs(data.controlled or {}) do
            if c.rb:getParentBody() ~= nil then
                ship:detach(c.rb)
                game.world:addRigidBody(c.rb) -- the scene teardown takes it out again
            end
        end
        if data.carrier then
            for _, child in ipairs(data.carrier.children) do
                data.carrier.rb:detach(child)
            end
        end
        if data.shipTrigger then
            data.shipTrigger:detach(ship)
            game.world:removeTrigger(data.shipTrigger)
        end
        if data.worldTrigger then
            game.world:removeTrigger(data.worldTrigger)
        end
        if data.wireframes ~= nil then
            GameState.debug.physics.drawWireframes = data.wireframes
        end
    end,
}
