-- Flies a ship past a belt rock that moves with an orbiting planet and checks that the ship's motion
-- relative to the rock is the same as when the planet stands still: a rock that moves with the ship's
-- gravity well is not moving as far as the ship is concerned. Both scenes share one physics world, far
-- apart, and run the systems in the order of the flight apps. Logs each failed check and ends with a
-- failure exit code.

local Application          = require("States.Application")
local Registry             = require("Core.ECS.Registry")
local Entity               = require("Core.ECS.Entity")
local CoreComponents       = require("Modules.Core.Components")
local PhysicsComponents    = require("Modules.Physics.Components")
local SpatialComponents    = require("Modules.Spatial.Components")
local SpatialEntities      = require("Modules.Spatial.Entities")
local CelestialComponents  = require("Modules.CelestialObjects.Components")
local CelestialEntities    = require("Modules.CelestialObjects.Entities")
local ConstructComponents  = require("Modules.Constructs.Components")
local AsteroidFieldSystem  = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local OrbitalSystem        = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem    = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem     = require("Modules.Constructs.Systems.ShipFlightSystem")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper  = require("Shared.Helpers.FlightProfileHelper")

---@class PhysicsBeltFlightTest: Application
local BeltFlightTest = Subclass("PhysicsBeltFlightTest", Application)

local STEP  = 1 / 60
local STEPS = 240

-- Game units. A rock of about a kilometre and a ship hull of about fifty metres.
local ROCK_RADIUS   = 0.1
local SHIP_RADIUS   = 0.0025
-- Distance between the rock's surface and the ship's at the start.
local START_GAP     = 0.01
-- The planet's orbit around the star, and the rock's offset from the planet.
local ORBIT_RADIUS  = 10000
local ROCK_OFFSET   = 200
local WELL_RADIUS   = 1000
-- The second scene is this far from the first, along x.
local SCENE_SPACING = 1e6
-- Control input held for the whole run: full forward thrust, and a turn toward the rock.
local THRUST        = 1.0
local YAW_TO_ROCK   = 0.2

local TOLERANCE = 1e-6

function BeltFlightTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("PhysicsBeltFlightTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A kinematic sphere of `radius` at `pos`, owned by `entity` and in the world.
function BeltFlightTest:giveBody(entity, pos, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(pos)
    return rb
end

---A planet, in a well, on a circular orbit around a star at `starPos`, with a belt of one rock. The
---planet stands still when `moving` is false.
function BeltFlightTest:buildPlanet(starPos, moving)
    local star = Entity.Create("StarEntity", CelestialComponents.Star())
    self:giveBody(star, starPos, 1)

    local planetPos = Position(starPos.x + ORBIT_RADIUS, starPos.y, starPos.z)
    local planet = Entity.Create("PlanetEntity", CelestialComponents.Planet())
    self:giveBody(planet, planetPos, 1)
    planet:add(PhysicsComponents.Transform()):setPos(planetPos)
    planet:add(SpatialComponents.Orbit(ORBIT_RADIUS))
    Registry:attachEntity(star, planet)

    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setRadius(WELL_RADIUS)
    Registry:attachEntity(planet, zone)

    local belt = CelestialEntities.AsteroidBelt(1)
    Registry:attachEntity(planet, belt)
    local lodMesh = LodMesh.Create()
    lodMesh:add(Mesh.Box(1), 0, 1e16)
    local rocks = { { px = ROCK_OFFSET, py = 0, pz = 0, scale = ROCK_RADIUS, rotSeed = 1 } }
    belt:add(CelestialComponents.AsteroidBeltData(rocks, ROCK_OFFSET, 0, lodMesh))

    local orbiters, followers = OrbitalSystem:collectOrbiters(star)
    if not moving then
        for _, orbiter in ipairs(orbiters) do orbiter.orbit:setAngularSpeed(0) end
    end
    return { planet = planet, belt = belt, orbiters = orbiters, followers = followers,
        rockPos = Position(planetPos.x + ROCK_OFFSET, planetPos.y, planetPos.z) }
end

---A Solo ship with a well and a control input, with the rock on its right, `START_GAP` from its hull.
function BeltFlightTest:buildShip(scene)
    local ship = Registry:createEntity()
    local rb = RigidBody.CreateSphere()
    rb:setScale(SHIP_RADIUS)
    local profile = FlightProfileRegistry:get(Enums.FlightProfile.Solo)
    FlightProfileHelper.applyToRigidBody(profile, rb)
    ship:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    self.world:addRigidBody(rb)

    local right = rb:getRot():getRight()
    local distance = ROCK_RADIUS + SHIP_RADIUS + START_GAP
    local rock = scene.rockPos
    rb:setPos(Position(rock.x - right.x * distance, rock.y - right.y * distance, rock.z - right.z * distance))

    ship:add(PhysicsComponents.GravityWell())
    ship:add(ConstructComponents.FlightProfile(profile))
    local input = ship:add(ConstructComponents.ControlInput())
    input:setThrustForward(THRUST)
    input:setYaw(YAW_TO_ROCK)
    scene.ship, scene.shipRb = ship, rb
end

---The rock the belt's system spawned, nil if none.
local function spawnedRock(scene)
    local children = scene.belt:get(CoreComponents.Children)
    if not children then return nil end
    for child in children:iterChildren() do
        return child:get(PhysicsComponents.RigidBody):getRigidBody()
    end
    return nil
end

---Runs the systems in the order of the flight apps' Sim handler.
function BeltFlightTest:step(scene)
    OrbitalSystem:update(scene.orbiters, STEP, scene.followers)
    AsteroidFieldSystem:updatePositions()
    GravityWellSystem:update()
    ShipFlightSystem:update()
end

---Where the ship is relative to the rock after each step, and the closest the ship's hull came to the
---rock's surface, over `STEPS` steps of both scenes.
function BeltFlightTest:run(scenes)
    local paths, closest = {}, {}
    for i, scene in ipairs(scenes) do
        paths[i], closest[i] = {}, math.huge
        scene.rock = assert(spawnedRock(scene), "the belt system did not spawn the rock")
    end

    for stepIndex = 1, STEPS do
        for _, scene in ipairs(scenes) do self:step(scene) end
        self.world:update(STEP)
        for i, scene in ipairs(scenes) do
            local ship, rock = scene.shipRb:getPos(), scene.rock:getPos()
            local rel = Position(ship.x - rock.x, ship.y - rock.y, ship.z - rock.z)
            paths[i][stepIndex] = rel
            local gap = math.sqrt(rel.x * rel.x + rel.y * rel.y + rel.z * rel.z) - ROCK_RADIUS - SHIP_RADIUS
            closest[i] = math.min(closest[i], gap)
        end
    end
    return paths, closest
end

---The largest distance between two paths at the same step, and the step it is at.
local function largestDeviation(a, b)
    local worst, at = 0, 0
    for i = 1, #a do
        local dx, dy, dz = a[i].x - b[i].x, a[i].y - b[i].y, a[i].z - b[i].z
        local d = math.sqrt(dx * dx + dy * dy + dz * dz)
        if d > worst then worst, at = d, i end
    end
    return worst, at
end

function BeltFlightTest:onInit()
    self.passed, self.failed = 0, 0
    -- The belt system gives each rock the asteroid material.
    require("Shared.Definitions.MaterialDefs")

    self.world = Physics.Create()
    self.world:setLengthUnit(Config.game.physics.lengthUnit)
    self.world:setContactNaturalFrequency(Config.game.physics.contactNaturalFrequency)

    local moving = self:buildPlanet(Position(0, 0, 0), true)
    local still = self:buildPlanet(Position(SCENE_SPACING, 0, 0), false)
    GravityWellSystem:collectZones()
    self:buildShip(moving)
    self:buildShip(still)
    self.world:update(STEP)
    -- A ship that enters a well is not moved by it on that update: enter now, so that the ships follow
    -- their planets from the first step of the run.
    GravityWellSystem:update()

    -- The system spawns the rocks near the ship it is given; one update per scene.
    AsteroidFieldSystem:update(1.0, { moving.belt }, self.world, moving.ship)
    AsteroidFieldSystem:update(1.0, { still.belt }, self.world, still.ship)

    local startX = moving.planet:get(PhysicsComponents.RigidBody):getRigidBody():getPos().x
    local paths, closest = self:run({ moving, still })
    local endX = moving.planet:get(PhysicsComponents.RigidBody):getRigidBody():getPos()
    local travelled = math.sqrt((endX.x - startX) ^ 2 + endX.y ^ 2 + endX.z ^ 2)
    local deviation, at = largestDeviation(paths[1], paths[2])

    Log.Info("PhysicsBeltFlightTest: planet moved %.3f, closest gap moving %.6f, still %.6f, deviation %.6g at step %d",
        travelled, closest[1], closest[2], deviation, at)

    self:check(travelled > ROCK_RADIUS, "the planet moves far enough to matter", tostring(travelled))
    self:check(closest[2] < SHIP_RADIUS, "the ship comes to the rock, so contacts are tested", tostring(closest[2]))
    self:check(deviation <= TOLERANCE, "the ship moves the same relative to a moving rock as to a still one",
        string.format("%.6g at step %d", deviation, at))

    AsteroidFieldSystem:cleanup(self.world)
    Log.Info("PhysicsBeltFlightTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return BeltFlightTest
