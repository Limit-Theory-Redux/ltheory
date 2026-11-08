-- Flies the player's fighter in circles around a belt rock of a still planet, as a pilot does by hand,
-- and checks that the rock changes nothing until the ship touches it: each scene has a twin whose belt
-- has no rock, flown by the same controller around the same point. A ship steered into the rock is
-- checked to press against it and stay outside it; its heading error, spin and speed in contact are
-- logged only. Runs the systems in the order of the flight apps. Logs each failed check and ends with
-- a failure exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local Entity              = require("Core.ECS.Entity")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local SpatialEntities     = require("Modules.Spatial.Entities")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local CelestialEntities   = require("Modules.CelestialObjects.Entities")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructManager    = require("Modules.Constructs.Managers.ConstructManager")
local AsteroidFieldSystem = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")

---@class PhysicsRockFlightTest: Application
local RockFlightTest = Subclass("PhysicsRockFlightTest", Application)

local STEP  = 1 / 60
local STEPS = 1200
-- Log every this many steps.
local LOG_EVERY = 60

-- Each variant is a rock radius and the circle the pilot holds around the rock's centre, given as its
-- gap to the rock's surface; a negative gap steers into the rock. Game units.
local VARIANTS = {
    { rockRadius = 0.1, circleGap = 0.5 },
    { rockRadius = 1,   circleGap = 0.5 },
    { rockRadius = 5,   circleGap = 0.5 },
    { rockRadius = 1,   circleGap = -0.05 },
    { rockRadius = 5,   circleGap = -0.05 },
}
-- The ship starts this far from the rock's surface, in game units.
local START_GAP     = 0.5
-- The planet's well, and the rock's offset from the planet, in game units.
local WELL_RADIUS   = 1000
local ROCK_OFFSET   = 200
-- Scenes are this far apart, along x.
local SCENE_SPACING = 1e5
-- The radial error, in game units, that bends the pilot's course fully inward or outward.
local RADIAL_RANGE  = 0.25
local STEER_GAIN    = 1.0
local THRUST        = 1.0
local SHIP_SEED     = 1234

local HULL      = Enums.ShipHulls.Solo
-- Paths whose largest difference stays below this are the same.
local TOLERANCE = 1e-6

function RockFlightTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- A warning, not Log.Error, which raises and would skip the checks after this one.
        Log.Warn("PhysicsRockFlightTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

local function dot(a, b) return a.x * b.x + a.y * b.y + a.z * b.z end
local function length(v) return math.sqrt(dot(v, v)) end

---A kinematic sphere of `radius` at `pos`, owned by `entity` and in the world.
function RockFlightTest:giveBody(entity, pos, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(pos)
    return rb
end

---A still planet in a well, with a belt of one rock of the variant's radius, or of none when `ghost`
---is set.
function RockFlightTest:buildPlanet(origin, variant, ghost)
    local planet = Entity.Create("PlanetEntity", CelestialComponents.Planet())
    self:giveBody(planet, origin, 1)
    planet:add(PhysicsComponents.Transform()):setPos(origin)

    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setRadius(WELL_RADIUS)
    Registry:attachEntity(planet, zone)

    local belt = CelestialEntities.AsteroidBelt(1)
    Registry:attachEntity(planet, belt)
    local lodMesh = LodMesh.Create()
    lodMesh:add(Mesh.Box(1), 0, 1e16)
    local rocks = {}
    if not ghost then
        rocks[1] = { px = ROCK_OFFSET, py = 0, pz = 0, scale = variant.rockRadius, rotSeed = 1 }
    end
    belt:add(CelestialComponents.AsteroidBeltData(rocks, ROCK_OFFSET, 0, lodMesh))

    return {
        belt = belt, rockRadius = variant.rockRadius, circleRadius = variant.rockRadius + variant.circleGap,
        ghost = ghost,
        centre = Position(origin.x + ROCK_OFFSET, origin.y, origin.z),
        name = string.format("%s r=%g gap=%g", ghost and "ghost" or "rock", variant.rockRadius, variant.circleGap),
    }
end

---The player's fighter, built and tuned as the flight app spawns it, on the side of the rock away
---from the planet.
function RockFlightTest:buildShip(scene)
    local c = scene.centre
    local handle = self.constructManager:createShip({
        seed = SHIP_SEED,
        shipType = Enums.ShipType.Fighter,
        config = {
            position = Position(c.x + scene.rockRadius + START_GAP, c.y, c.z),
            scale = Config.game.shipHulls.scale[HULL],
            isKinematic = false,
            flightProfile = Enums.FlightProfile.Solo,
        },
    })
    local ship = handle.root
    ship:add(ConstructComponents.TravelDrive())
    scene.input = ship:add(ConstructComponents.ControlInput())
    ship:add(PhysicsComponents.GravityWell(ConstructComponents.FlightProfile.ProfileOf(ship).linearDrag))
    scene.ship, scene.shipRb = ship, handle.rigidBody
    -- The hull's bounding sphere: contact is counted when it touches, a little before the hull does.
    scene.shipRadius = handle.rigidBody:getBoundingRadiusCompound()
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

---A pilot's input for circling the centre at the scene's circle radius in the plane y = centre.y:
---steer toward the tangent, bent inward or outward by the radial error, with full forward thrust.
---Keeps the direction steered toward in `scene.desired`.
local function circleInput(scene)
    local rb, input = scene.shipRb, scene.input
    local p, c = rb:getPos(), scene.centre
    local rx, rz = p.x - c.x, p.z - c.z
    local r = math.sqrt(rx * rx + rz * rz)
    rx, rz = rx / r, rz / r
    -- Counter-clockwise tangent seen from above, plus a radial correction.
    local bend = Math.Clamp(-(r - scene.circleRadius) / RADIAL_RANGE, -1, 1)
    local dx, dy, dz = -rz + rx * bend, (c.y - p.y) / RADIAL_RANGE, rx + rz * bend
    local dlen = math.sqrt(dx * dx + dy * dy + dz * dz)
    scene.desired = Vec3f(dx / dlen, dy / dlen, dz / dlen)

    local rot = rb:getRot()
    input:setThrustForward(THRUST)
    input:setYaw(Math.Clamp(STEER_GAIN * dot(scene.desired, rot:getRight()), -1, 1))
    input:setPitch(Math.Clamp(STEER_GAIN * dot(scene.desired, rot:getUp()), -1, 1))
end

---One frame in the order of the flight apps: PreSim writes the input, Sim moves the bodies, actuates the
---ships and steps physics.
function RockFlightTest:step(scenes)
    for _, scene in ipairs(scenes) do
        AsteroidFieldSystem:update(STEP, { scene.belt }, self.world, scene.ship)
        circleInput(scene)
    end
    AsteroidFieldSystem:updatePositions()
    GravityWellSystem:update(STEP, self.world:getSolverSubsteps())
    TravelDriveSystem:update(STEP)
    ShipFlightSystem:update()
    self.world:update(STEP)
end

---The gap between the ship's hull and the rock's surface.
local function gapOf(scene)
    local p, c = scene.shipRb:getPos(), scene.centre
    local d = math.sqrt((p.x - c.x) ^ 2 + (p.y - c.y) ^ 2 + (p.z - c.z) ^ 2)
    return d - scene.rockRadius - scene.shipRadius
end

---The angle, in degrees, between the ship's nose and the direction the pilot steered toward.
local function headingError(scene)
    local fwd = scene.shipRb:getRot():getForward()
    return math.deg(math.acos(Math.Clamp(dot(fwd, scene.desired), -1, 1)))
end

---Updates the scene's contact statistics with this step: steps in contact, the largest heading error
---and spin then, and the sum of the speeds then.
local function recordContact(scene, gap, stepIndex)
    if gap > 0 then return end
    if not scene.contactStep then scene.contactStep = stepIndex end
    scene.contactSteps = scene.contactSteps + 1
    -- Past the first second of contact: the ship had time to turn onto its pressed heading.
    if stepIndex - scene.contactStep >= LOG_EVERY then
        scene.pressedError = math.max(scene.pressedError, headingError(scene))
    end
    scene.contactSpin = math.max(scene.contactSpin, length(scene.shipRb:getVelocityA()))
    scene.contactSpeedSum = scene.contactSpeedSum + length(scene.shipRb:getVelocity())
end

---Records each scene's path relative to its centre, its closest gap, its contacts, and the rock's
---largest speed and spin.
function RockFlightTest:run(scenes)
    for _, scene in ipairs(scenes) do
        scene.path, scene.closest, scene.contactStep = {}, math.huge, nil
        scene.rockSpeed, scene.rockSpin = 0, 0
        scene.contactSteps, scene.pressedError, scene.contactSpin, scene.contactSpeedSum = 0, 0, 0, 0
    end
    for stepIndex = 1, STEPS do
        self:step(scenes)
        for _, scene in ipairs(scenes) do
            local p, c = scene.shipRb:getPos(), scene.centre
            scene.path[stepIndex] = Position(p.x - c.x, p.y - c.y, p.z - c.z)
            local gap = gapOf(scene)
            scene.closest = math.min(scene.closest, gap)
            if scene.rock then
                recordContact(scene, gap, stepIndex)
                scene.rockSpeed = math.max(scene.rockSpeed, length(scene.rock:getVelocity()))
                scene.rockSpin = math.max(scene.rockSpin, length(scene.rock:getVelocityA()))
            end
            if stepIndex % LOG_EVERY == 0 then
                Log.Info("PhysicsRockFlightTest: %-20s step %4d gap %9.5f speed %.4f spin %.4f heading error %6.2f yaw %+.3f pitch %+.3f",
                    scene.name, stepIndex, gap, length(scene.shipRb:getVelocity()),
                    length(scene.shipRb:getVelocityA()), headingError(scene),
                    scene.input:getYaw(), scene.input:getPitch())
            end
        end
    end
end

---The largest distance between two paths up to step `last`, and the step it is at.
local function largestDeviation(a, b, last)
    local worst, at = 0, 0
    for i = 1, last do
        local d = math.sqrt((a[i].x - b[i].x) ^ 2 + (a[i].y - b[i].y) ^ 2 + (a[i].z - b[i].z) ^ 2)
        if d > worst then worst, at = d, i end
    end
    return worst, at
end

---Checks one rock scene against its ghost.
function RockFlightTest:checkPair(rock, ghost)
    local last = (rock.contactStep or STEPS + 1) - 1
    local deviation, at = largestDeviation(rock.path, ghost.path, last)
    local meanSpeed = rock.contactSteps > 0 and rock.contactSpeedSum / rock.contactSteps or 0
    Log.Info("PhysicsRockFlightTest: %s: ship radius %.5f, closest gap %.5f (ghost %.5f), first contact %s, deviation before it %.6g at step %d, rock speed %.3g spin %.3g",
        rock.name, rock.shipRadius, rock.closest, ghost.closest, tostring(rock.contactStep), deviation, at,
        rock.rockSpeed, rock.rockSpin)
    Log.Info("PhysicsRockFlightTest: %s: %d steps in contact, heading error pressed %.2f deg, spin up to %.4f, mean speed %.4f",
        rock.name, rock.contactSteps, rock.pressedError, rock.contactSpin, meanSpeed)

    self:check(rock.rockSpeed == 0 and rock.rockSpin == 0, rock.name .. ": the rock stays still",
        string.format("speed %.3g, spin %.3g", rock.rockSpeed, rock.rockSpin))
    self:check(deviation <= TOLERANCE, rock.name .. ": the rock changes nothing before the ship touches it",
        string.format("%.6g at step %d", deviation, at))
    if rock.circleRadius > rock.rockRadius then
        self:check(ghost.closest > 0, rock.name .. ": the pilot's circle stays off the surface",
            tostring(ghost.closest))
        self:check(rock.closest > 0, rock.name .. ": the ship circling the rock never touches it",
            string.format("closest %.6f, first contact at step %s", rock.closest, tostring(rock.contactStep)))
    else
        self:check(rock.contactSteps > LOG_EVERY, rock.name .. ": the pilot presses the ship against the rock",
            tostring(rock.contactSteps))
        self:check(rock.closest > -rock.shipRadius, rock.name .. ": the ship stays outside the rock",
            tostring(rock.closest))
    end
end

function RockFlightTest:onInit()
    self.passed, self.failed = 0, 0
    -- The belt system gives each rock the asteroid material.
    require("Shared.Definitions.MaterialDefs")

    self.world = Physics.Create()
    self.world:setLengthUnit(Config.game.physics.lengthUnit)
    self.world:setContactNaturalFrequency(Config.game.physics.contactNaturalFrequency)
    self.constructManager = ConstructManager(Registry, self.world)

    local pairsOfScenes, scenes = {}, {}
    for i, variant in ipairs(VARIANTS) do
        local rock = self:buildPlanet(Position((2 * i - 2) * SCENE_SPACING, 0, 0), variant, false)
        local ghost = self:buildPlanet(Position((2 * i - 1) * SCENE_SPACING, 0, 0), variant, true)
        pairsOfScenes[i] = { rock = rock, ghost = ghost }
        scenes[#scenes + 1] = rock
        scenes[#scenes + 1] = ghost
    end
    GravityWellSystem:collectZones()
    for _, scene in ipairs(scenes) do self:buildShip(scene) end
    self.world:update(STEP)
    GravityWellSystem:update(STEP, self.world:getSolverSubsteps())

    -- The system spawns rocks near the ship it is given, once its update interval has passed.
    for _, scene in ipairs(scenes) do
        AsteroidFieldSystem:update(1.0, { scene.belt }, self.world, scene.ship)
        scene.rock = spawnedRock(scene)
        self:check((scene.rock ~= nil) ~= scene.ghost, scene.name .. ": the belt spawns its rock, and only then")
    end

    self:run(scenes)
    for _, pair in ipairs(pairsOfScenes) do self:checkPair(pair.rock, pair.ghost) end

    AsteroidFieldSystem:cleanup(self.world)
    Log.Info("PhysicsRockFlightTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return RockFlightTest
