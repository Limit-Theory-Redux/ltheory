-- Checks that the autopilot flies a ship through its control input: it arrives at a point and at
-- a body, with and without the travel drive, stops outside the body's hull, steers around a rock on
-- its course, and leaves no axes behind. It also arrives at a planet whose orbit is faster than the
-- ship's top speed, from inside the planet's gravity well, which carries the ship with the planet.
-- A drive flight among rocks logs the time the autopilot's update takes with and without sensing.
-- Logs each failed check and ends with a failure exit code.
local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local PhysicsComponents     = require("Modules.Physics.Components")
local SpatialComponents     = require("Modules.Spatial.Components")
local SpatialEntities       = require("Modules.Spatial.Entities")
local CelestialComponents   = require("Modules.CelestialObjects.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local AutoPilotSystem       = require("Modules.Constructs.Systems.AutoPilotSystem")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper   = require("Shared.Helpers.FlightProfileHelper")

---@class ConstructsAutoPilotTest: Application
local AutoPilotTest = Subclass("ConstructsAutoPilotTest", Application)

local STEP         = 1 / 60
local HULL         = Enums.ShipHulls.Solo
local SOLO         = Enums.FlightProfile.Solo
-- Bound on the time a ship takes to stop once the autopilot has let go: it turns a ship that never
-- stops into a failure.
local MAX_SETTLE_S = 120
-- The ETA at the moment the drive runs may differ from the flight that remains by this share. It
-- leaves out the drive's ramp-up, so it runs short.
local ETA_TOLERANCE = 0.2
-- The ship has stopped below this speed, in game units per second.
local STOPPED      = 1e-3
-- Where the targets are: mostly behind the ship and to one side, so that it has to turn.
local DIRECTION    = { x = 0.6, y = 0.3, z = 0.74 }
local BODY_RADIUS  = 100
local ROCK_RADIUS  = 5
-- The rocks of the field: how many, and how far from the course they lie, in game units. The
-- field's count is the most the asteroid system keeps spawned at once.
local FIELD_ROCKS     = 100
local FIELD_MIN_SIDE  = 100
local FIELD_SIDE_STEP = 50
local FIELD_SIDE_KINDS = 5
-- The rocks of the field lie between these shares of the distance to the target.
local FIELD_FIRST_ALONG = 0.05
local FIELD_ALONG_SPAN  = 0.9
-- The angle between the sides of consecutive rocks, around the course (the golden angle).
local FIELD_TURN = math.pi * (3 - math.sqrt(5))

-- The moving well: a planet on a circular orbit around a star, with a gravity well, in game units.
-- The planet's speed on its orbit is this many times the ship's top speed, so a ship that steers
-- against the carry can't keep up with it.
local WELL_ORBIT_RADIUS  = 10000
local WELL_RADIUS        = 1000
local WELL_ORBIT_SPEEDUP = 2

---@class AutoPilotRock
---@field along number Share of the distance to the target, along the straight course from the start
---@field sideA gameunit Offset from the course along the first axis across it
---@field sideB gameunit Offset from the course along the second axis across it
---@field radius gameunit

---@class AutoPilotScenario
---@field name string
---@field distance gameunit Distance of the target from the ship's start
---@field radius gameunit|nil Radius of a target body; nil for a bare point
---@field drive boolean Whether the travel drive is expected to engage
---@field maxFlight number Seconds of simulated time the autopilot may take to arrive: about 1.2 times what it takes
---@field rocks AutoPilotRock[]|nil Fixed spheres in the ship's way or beside it
---@field sensing boolean|nil False turns the autopilot's sensing off; the run is the baseline for its cost
---@field timed boolean|nil Whether the run's update times are logged, as a pair with `sensing` on and off
---@field well boolean|nil The target is a planet in a moving gravity well, and the ship starts inside the well

---Rocks spread around the course, clear of it, so the whole field is within the sensing range of a
---ship at drive speed but none is in its way (the drive's avoidance is not tested).
---@return AutoPilotRock[]
local function rockField()
    local rocks = {}
    for index = 0, FIELD_ROCKS - 1 do
        local side = FIELD_MIN_SIDE + (index % FIELD_SIDE_KINDS) * FIELD_SIDE_STEP
        local turn = index * FIELD_TURN
        rocks[#rocks + 1] = {
            along  = FIELD_FIRST_ALONG + FIELD_ALONG_SPAN * index / FIELD_ROCKS,
            sideA  = side * math.cos(turn),
            sideB  = side * math.sin(turn),
            radius = ROCK_RADIUS,
        }
    end
    return rocks
end

---@type AutoPilotScenario[]
local SCENARIOS = {
    { name = "a point, no drive",  distance = 300,   drive = false, maxFlight = 300 },
    { name = "a body, no drive",   distance = 600,   radius = BODY_RADIUS, drive = false, maxFlight = 570 },
    { name = "a body, with drive", distance = 20000, radius = BODY_RADIUS, drive = true,  maxFlight = 40 },
    { name = "a point past a rock", distance = 300, drive = false, maxFlight = 400,
      rocks = { { along = 0.5, sideA = 0, sideB = 0, radius = ROCK_RADIUS } } },
    { name = "a body past a rock", distance = 600, radius = BODY_RADIUS, drive = false, maxFlight = 700,
      rocks = { { along = 0.5, sideA = 0, sideB = 0, radius = ROCK_RADIUS } } },
    { name = "a body, with drive, among rocks", distance = 20000, radius = BODY_RADIUS, drive = true,
      maxFlight = 40, rocks = rockField(), timed = true },
    { name = "a body, with drive, among rocks, sensing off", distance = 20000, radius = BODY_RADIUS,
      drive = true, maxFlight = 40, rocks = rockField(), sensing = false, timed = true },
    { name = "a body in a moving well", distance = 600, radius = BODY_RADIUS, drive = false, maxFlight = 700,
      well = true },
}

function AutoPilotTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("ConstructsAutoPilotTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A ship set up like the player ship, at `start`, with the autopilot component and, when `inWell`,
---a gravity well component that lets a zone carry it.
---@param start Position
---@param inWell boolean|nil
function AutoPilotTest:spawnShip(start, inWell)
    local profile = FlightProfileRegistry:get(SOLO)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(Config.game.shipHulls.scale[HULL])
    FlightProfileHelper.applyToRigidBody(profile, rb)
    entity:add(ConstructComponents.FlightProfile(profile))
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(start)
    if inWell then entity:add(PhysicsComponents.GravityWell(profile.linearDrag)) end
    entity:add(ConstructComponents.TravelDrive())
    entity:add(ConstructComponents.ControlInput())
    entity:add(ConstructComponents.AutoPilot())
    return { entity = entity, rb = rb }
end

---A kinematic sphere of `radius` at `pos`, owned by `entity` and in the world.
local function giveKinematicBody(world, entity, pos, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    world:addRigidBody(rb)
    rb:setPos(pos)
    return rb
end

---A planet of `BODY_RADIUS` on a circular orbit around a star, with a gravity well, whose speed on
---its orbit is `WELL_ORBIT_SPEEDUP` times the Solo top speed.
---@return table well `planetBody` is the planet as a target: its entity and rigid body
function AutoPilotTest:buildWell()
    local star = Entity.Create("StarEntity", CelestialComponents.Star())
    local starRb = giveKinematicBody(self.world, star, Position(0, 0, 0), 1)

    local planetPos = Position(WELL_ORBIT_RADIUS, 0, 0)
    local planet = Entity.Create("PlanetEntity", CelestialComponents.Planet())
    local planetRb = giveKinematicBody(self.world, planet, planetPos, BODY_RADIUS)
    planet:add(PhysicsComponents.Transform()):setPos(planetPos)
    planet:add(SpatialComponents.Orbit(WELL_ORBIT_RADIUS))
    Registry:attachEntity(star, planet)

    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setRadius(WELL_RADIUS)
    Registry:attachEntity(planet, zone)

    local orbiters, followers = OrbitalSystem:collectOrbiters(star)
    local topSpeed = FlightProfileRegistry:get(SOLO).maxSpeed
    for _, orbiter in ipairs(orbiters) do
        orbiter.orbit:setAngularSpeed(WELL_ORBIT_SPEEDUP * topSpeed / WELL_ORBIT_RADIUS)
    end
    GravityWellSystem:collectZones()
    return {
        star = star, starRb = starRb, zone = zone, orbiters = orbiters, followers = followers,
        planetPos = planetPos, planetBody = { entity = planet, rb = planetRb },
    }
end

---Remove a moving well's bodies from physics and destroy its entities.
function AutoPilotTest:despawnWell(well)
    self.world:removeRigidBody(well.starRb)
    Registry:destroyEntity(well.zone)
    Registry:destroyEntity(well.star)
    GravityWellSystem:collectZones()
end

---A fixed sphere of `radius`, the target of a body scenario.
function AutoPilotTest:spawnBody(position, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    local entity = Entity.Create("AutoPilotTestBody", PhysicsComponents.RigidBody(rb))
    self.world:addRigidBody(rb)
    return { entity = entity, rb = rb }
end

---The smallest gap between the ship's hull and a rock's surface, or infinity without rocks.
---@param ship table
---@param rocks table[]
---@return number
function AutoPilotTest:closestRockGap(ship, rocks)
    local gap = math.huge
    local shipPos = ship.rb:getPos()
    local hull = ship.rb:getBoundingRadius()
    for _, rock in ipairs(rocks) do
        gap = math.min(gap, shipPos:distance(rock.rb:getPos()) - rock.rb:getBoundingRadius() - hull)
    end
    return gap
end

---Fixed spheres along the straight course to the target, or beside it.
---@param rocks AutoPilotRock[]
---@param course table Unit vector from the ship's start to the target
---@param distance gameunit
---@return table[] spawned
function AutoPilotTest:spawnRocks(rocks, course, distance)
    -- Two unit vectors across the course: the course crossed with the world's up, and the course
    -- crossed with that.
    local acrossLength = math.sqrt(course.x ^ 2 + course.z ^ 2)
    local a = { x = -course.z / acrossLength, y = 0, z = course.x / acrossLength }
    local b = {
        x = course.y * a.z - course.z * a.y,
        y = course.z * a.x - course.x * a.z,
        z = course.x * a.y - course.y * a.x,
    }
    local spawned = {}
    for _, rock in ipairs(rocks) do
        local along = rock.along * distance
        spawned[#spawned + 1] = self:spawnBody(Position(
            course.x * along + a.x * rock.sideA + b.x * rock.sideB,
            course.y * along + a.y * rock.sideA + b.y * rock.sideB,
            course.z * along + a.z * rock.sideA + b.z * rock.sideB), rock.radius)
    end
    return spawned
end

---Remove a scenario's bodies from physics and destroy their entities, so the actuation systems
---no longer see them.
function AutoPilotTest:despawn(ship, body, rocks)
    local things = { ship, body or false }
    for _, rock in ipairs(rocks) do
        things[#things + 1] = rock
    end
    for _, thing in ipairs(things) do
        if thing then
            self.world:removeRigidBody(thing.rb)
            Registry:destroyEntity(thing.entity)
        end
    end
end

---One simulation step in the order the apps use: the autopilot at PreSim, then the drive, the orbits
---and wells of a moving well, flight and physics.
---@param ship table
---@param well table|nil The scenario's moving well
---@return number updateMs Milliseconds the autopilot's update took
function AutoPilotTest:step(ship, well)
    local start = InstantTime.Now()
    AutoPilotSystem:update(STEP, ship.entity)
    local updateMs = InstantTime.Now():durationSince(start)
    TravelDriveSystem:update(STEP)
    if well then
        OrbitalSystem:update(well.orbiters, STEP, well.followers)
        GravityWellSystem:update(STEP, self.world:getSolverSubsteps())
    end
    ShipFlightSystem:update()
    self.world:update(STEP)
    return updateMs
end

local function axesAreZero(input)
    return input:getThrustForward() == 0 and input:getThrustRight() == 0 and input:getThrustUp() == 0
        and input:getPitch() == 0 and input:getYaw() == 0 and input:getRoll() == 0
end

---Fly one scenario and check it.
---@param scenario AutoPilotScenario
function AutoPilotTest:run(scenario)
    local name = scenario.name .. ": "
    local length = math.sqrt(DIRECTION.x ^ 2 + DIRECTION.y ^ 2 + DIRECTION.z ^ 2)
    local well = scenario.well and self:buildWell()
    -- A ship starts at the origin, or at the distance of the scenario from a planet's start.
    local origin = well and Position(
        well.planetPos.x - DIRECTION.x / length * scenario.distance,
        well.planetPos.y - DIRECTION.y / length * scenario.distance,
        well.planetPos.z - DIRECTION.z / length * scenario.distance) or Position(0, 0, 0)
    local targetPos = Position(
        origin.x + DIRECTION.x / length * scenario.distance,
        origin.y + DIRECTION.y / length * scenario.distance,
        origin.z + DIRECTION.z / length * scenario.distance)

    local ship = self:spawnShip(origin, well ~= nil)
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    local ap = ship.entity:get(ConstructComponents.AutoPilot)
    local body = well and well.planetBody or (scenario.radius and self:spawnBody(targetPos, scenario.radius))
    -- Where the target is now: it moves in a moving well.
    local function aim() return body and body.rb:getPos() or targetPos end
    local rocks = self:spawnRocks(scenario.rocks or {}, {
        x = DIRECTION.x / length, y = DIRECTION.y / length, z = DIRECTION.z / length,
    }, scenario.distance)
    -- Sensing reads the query index as the last physics update left it, so the rocks need one.
    if #rocks > 0 then self.world:update(STEP) end
    -- Enter the well now, so that the ship is carried with the planet from the first step.
    if well then
        self.world:update(STEP)
        GravityWellSystem:update(STEP, self.world:getSolverSubsteps())
    end
    if body then
        AutoPilotSystem:engageEntity(ship.entity, self.world, body.entity)
    else
        AutoPilotSystem:engagePosition(ship.entity, self.world, targetPos)
    end
    if scenario.sensing == false then
        ap:getNav():setSensing(0)
    end
    local targetRadius = scenario.radius or 0
    local hull = ship.rb:getBoundingRadius()
    self:check(ap:isActive(), name .. "engaging activates the autopilot")

    -- The first update writes the axes of a ship that is far from its goal and not facing it.
    AutoPilotSystem:update(STEP, ship.entity)
    self:check(not axesAreZero(input), name .. "the autopilot writes the control input")

    local closest, driveSeen, arrivedAt = math.huge, false, nil
    local rockClosest = math.huge
    local etaAtDrive, driveAt
    local updateSum, updateMax, updateCount = 0, 0, 0
    for stepIndex = 1, math.floor(scenario.maxFlight / STEP) do
        local updateMs = self:step(ship, well)
        updateSum, updateCount = updateSum + updateMs, updateCount + 1
        updateMax = math.max(updateMax, updateMs)
        closest = math.min(closest, ship.rb:getPos():distance(aim()))
        rockClosest = math.min(rockClosest, self:closestRockGap(ship, rocks))
        if drive:isActive() and not driveSeen then
            driveSeen, etaAtDrive, driveAt = true, ap.eta, stepIndex * STEP
        end
        if not ap:isActive() then
            arrivedAt = stepIndex * STEP
            break
        end
    end
    self:check(arrivedAt ~= nil, name .. "the autopilot arrives", string.format("%.0f s", scenario.maxFlight))
    if not arrivedAt then
        self:despawn(ship, body, rocks)
        if well then self:despawnWell(well) end
        return
    end
    if etaAtDrive then
        local left = arrivedAt - driveAt
        self:check(math.abs(etaAtDrive - left) <= ETA_TOLERANCE * left, name .. "the ETA matches the flight",
            string.format("eta %.1f s, left %.1f s", etaAtDrive, left))
    end

    local standOff = AutoPilotSystem.standOff(targetRadius)
    local arrivalRadius = AutoPilotSystem.arrivalRadius(targetRadius)
    local distance = ship.rb:getPos():distance(aim())
    self:check(math.abs(distance - standOff) <= arrivalRadius + hull, name .. "it arrives at the stand-off distance",
        string.format("%.1f against %.1f ± %.1f", distance, standOff, arrivalRadius))
    self:check(axesAreZero(input), name .. "disengaging leaves no axes")
    self:check(driveSeen == scenario.drive, name .. "the travel drive engages only when it should",
        string.format("drive seen %s", tostring(driveSeen)))

    -- It stops outside the hull, once the drive and the drag have run out.
    for _ = 1, math.floor(MAX_SETTLE_S / STEP) do
        if ShipFlightSystem:getSpeed(ship.entity) < STOPPED and drive:getState() == "idle" then break end
        self:step(ship, well)
        closest = math.min(closest, ship.rb:getPos():distance(aim()))
        rockClosest = math.min(rockClosest, self:closestRockGap(ship, rocks))
    end
    -- At rest relative to the well that carries it (its world speed in open space).
    local restSpeed = ShipFlightSystem:getSpeed(ship.entity)
    self:check(restSpeed < STOPPED, name .. "the ship comes to rest", tostring(restSpeed))
    self:check(drive:getState() == "idle", name .. "the drive is idle", drive:getState())
    self:check(closest > targetRadius + hull, name .. "the ship stays outside the target's hull",
        string.format("closest %.1f, hull %.1f", closest, targetRadius + hull))
    Log.Info("ConstructsAutoPilotTest: %sarrived after %.0f s at %.0f (stand-off %.0f), closest %.0f, drive %s, eta at drive %s",
        name, arrivedAt, distance, standOff, closest, tostring(driveSeen),
        etaAtDrive and string.format("%.1f s, left %.1f s", etaAtDrive, arrivedAt - driveAt) or "-")
    if #rocks > 0 then
        self:check(rockClosest > 0, name .. "the ship never touches a rock",
            string.format("closest gap %.4f", rockClosest))
        Log.Info("ConstructsAutoPilotTest: %sclosest gap to a rock %.3f (hull %.3f)", name, rockClosest, hull)
    end
    if scenario.timed then
        local mean = updateSum / updateCount
        self.updateMeans[scenario.sensing == false and "off" or "on"] = mean
        Log.Info("ConstructsAutoPilotTest: %sautopilot update per frame: mean %.4f ms, max %.4f ms over %d frames",
            name, mean, updateMax, updateCount)
    end

    self:despawn(ship, body, rocks)
    if well then self:despawnWell(well) end
end

function AutoPilotTest:onInit()
    self.passed, self.failed = 0, 0
    -- Mean autopilot update time of the timed runs, in milliseconds, by whether sensing was on.
    self.updateMeans = {}

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    for _, scenario in ipairs(SCENARIOS) do
        self:run(scenario)
    end

    local means = self.updateMeans
    if means.on and means.off then
        Log.Info("ConstructsAutoPilotTest: sensing costs %.4f ms per frame in the autopilot update (%.4f on, %.4f off)",
            means.on - means.off, means.on, means.off)
    end

    Log.Info("ConstructsAutoPilotTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return AutoPilotTest
