-- Checks that the autopilot flies a ship through its control input: it arrives at a point and at
-- a body, with and without the travel drive, stops outside the body's hull, steers around a rock on
-- its course, and leaves no axes behind. It also arrives at a planet whose orbit is faster than the
-- ship's top speed, from inside the planet's gravity well, which carries the ship with the planet,
-- and from outside it, where the travel drive runs on through the well's entry to the stand-off point.
-- A drive flight among rocks logs the time the autopilot's update takes with and without sensing.
-- Logs each failed check and ends with a failure exit code.
local Application           = require("States.Application")
local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local AutoPilotSystem       = require("Modules.Constructs.Systems.AutoPilotSystem")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local PlanFlight            = require("Shared.Helpers.PlanFlight")
local FlightScene           = require("States.App.Tests.Support.FlightScene")

---@class ConstructsAutoPilotTest: Application
local AutoPilotTest = Subclass("ConstructsAutoPilotTest", Application)

local STEP         = 1 / 60
-- Bound on the time a ship takes to stop once the autopilot has let go: it turns a ship that never
-- stops into a failure.
local MAX_SETTLE_S = 120
-- The ETA at the moment the drive runs may differ from the flight that remains by this share. A
-- flight without a plan estimates the drive's time without its ramp-up, so it runs short; a plan's
-- ETA is the time the planner gives the ship to come within the arrival radius.
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
local WELL_ORBIT_RADIUS  = FlightScene.WELL_ORBIT_RADIUS
local WELL_ORBIT_SPEEDUP = 2
-- The planet of a drive flight into a well, as fast as the game's: a generated planet moves at
-- `0.5 · √(orbit radius)` game units per second (1,000 to 10,000), and the drive brings a ship to the
-- edge of its well at 2,000 to 9,000 (the braking law's speed for the way left, at most the zone's
-- cap), so the planet moves at 0.1 to 1 times the ship's speed there. Here the ship's speed at the
-- entry is about 200 game units per second, and the planet moves at about half of it: 115 times
-- the Solo top speed.
local WELL_FAST_SPEEDUP = 115
-- The nose's turn is judged over this many seconds around the ship's entry into the well.
local EDGE_WINDOW_S = 3
-- The most the nose may swing out and back across that window, in radians (about 9 degrees): a
-- swing a pilot does not see as a wrong turn. The slow planet's flight swings 0.05.
local MAX_EDGE_SWING = 0.15
-- The planet in the drive's way: its hull radius, how far along the course it lies as a share of
-- the distance, and its speed on its orbit as a share of the ship's top speed (slow: it stays across
-- the course while the ship flies).
local PLANET_RADIUS      = 500
local PLANET_ALONG       = 0.5
local PLANET_ORBIT_SPEEDUP = 0.01
-- The planner keeps the ship this many hull radii from a planet; the ship is only checked to stay
-- outside the hull, and the clearance from this sphere is logged.
local PLANET_SAFE_FACTOR = Config.game.autoPilot.safeRadiusFactor.planet
-- The wall-clock bound on waiting for a plan, in seconds: it turns a plan that never comes into a failure.
local PLAN_WAIT_S        = 30
local MS_PER_S           = 1000

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
---@field planet boolean|nil A planet with a gravity well lies across the course, on a slow orbit
---@field outside boolean|nil With `well`: the ship starts outside the well and has to be planned into it
---@field wellSpeedup number|nil The planet's speed on its orbit as a multiple of the ship's top speed; `WELL_ORBIT_SPEEDUP` when nil

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
    { name = "a point, with drive, past a planet", distance = 20000, drive = true, maxFlight = 140, planet = true },
    { name = "a body in a moving well, with drive, from outside", distance = 20000, radius = BODY_RADIUS,
      drive = true, maxFlight = 32, well = true, outside = true },
    { name = "a body in a moving well, with drive, from outside, a planet as fast as the game's", distance = 20000,
      radius = BODY_RADIUS, drive = true, maxFlight = 60, well = true, outside = true, wellSpeedup = WELL_FAST_SPEEDUP },
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
    return FlightScene.spawnShip(self.world, start, { inWell = inWell, autoPilot = true })
end

---A planet of `radius` at `planetPos` on a circular orbit around a star, with a gravity well, whose
---speed on its orbit is `speedup` times the Solo top speed.
---@param planetPos Position
---@param radius gameunit
---@param speedup number
---@return table well `planetBody` is the planet as a target: its entity and rigid body
function AutoPilotTest:buildWell(planetPos, radius, speedup)
    return FlightScene.buildWell(self.world, planetPos, radius, speedup)
end

---Remove a moving well's bodies from physics and destroy its entities.
function AutoPilotTest:despawnWell(well)
    FlightScene.despawnWell(self.world, well)
end

---A fixed sphere of `radius`, the target of a body scenario.
function AutoPilotTest:spawnBody(position, radius)
    return FlightScene.spawnBody(self.world, position, radius)
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
    local things = { ship }
    if body then things[#things + 1] = body end
    for _, rock in ipairs(rocks) do
        things[#things + 1] = rock
    end
    FlightScene.despawn(self.world, things)
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

---The angle between two vectors in radians, zero for a vector of no length.
local function angleBetween(a, b)
    local la = math.sqrt(a.x * a.x + a.y * a.y + a.z * a.z)
    local lb = math.sqrt(b.x * b.x + b.y * b.y + b.z * b.z)
    if la == 0 or lb == 0 then return 0 end
    local cosine = (a.x * b.x + a.y * b.y + a.z * b.z) / (la * lb)
    return math.acos(math.max(-1, math.min(1, cosine)))
end

---How far the nose swings out and back across a window of samples: the largest, over the window,
---of the smaller of its angles to the nose at the start and at the end. A nose that turns once,
---from one heading to another, stays near zero; one that turns away and returns is as large as the
---swing.
---@param samples table[] Samples with a `nose`, in time order
---@return number radians
local function noseExcursion(samples)
    local first, last = samples[1].nose, samples[#samples].nose
    local excursion = 0
    for _, sample in ipairs(samples) do
        excursion = math.max(excursion, math.min(angleBetween(sample.nose, first), angleBetween(sample.nose, last)))
    end
    return excursion
end

---The samples within `EDGE_WINDOW_S` of `at`.
local function samplesAround(samples, at)
    local window = {}
    for _, sample in ipairs(samples) do
        if math.abs(sample.at - at) <= EDGE_WINDOW_S then window[#window + 1] = sample end
    end
    return window
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
    local well = scenario.well and self:buildWell(Position(WELL_ORBIT_RADIUS, 0, 0), BODY_RADIUS,
        scenario.wellSpeedup or WELL_ORBIT_SPEEDUP)
    -- The planet across the course: the ship starts at the origin, so the course runs from there.
    local planet = scenario.planet and self:buildWell(Position(
        DIRECTION.x / length * scenario.distance * PLANET_ALONG,
        DIRECTION.y / length * scenario.distance * PLANET_ALONG,
        DIRECTION.z / length * scenario.distance * PLANET_ALONG), PLANET_RADIUS, PLANET_ORBIT_SPEEDUP)
    -- The scenario's moving bodies, whose orbits and wells are updated with each step.
    local moving = well or planet
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
    if #rocks > 0 or planet then self.world:update(STEP) end
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

    -- The plan comes from a worker: wait for it, so that every run flies the same way. The ship is
    -- not stepped, and `update` with no time passing only polls.
    local waitStart = InstantTime.Now()
    while AutoPilotSystem:isPlanning(ship.entity)
        and InstantTime.Now():durationSince(waitStart) < PLAN_WAIT_S * MS_PER_S do
        AutoPilotSystem:update(0, ship.entity)
    end
    self:check(not AutoPilotSystem:isPlanning(ship.entity), name .. "the plan comes")

    -- The first update writes the axes of a ship that is far from its goal and not facing it.
    AutoPilotSystem:update(STEP, ship.entity)
    self:check(not axesAreZero(input), name .. "the autopilot writes the control input")

    local closest, driveSeen, arrivedAt = math.huge, false, nil
    local rockClosest = math.huge
    local etaAtDrive, driveAt
    local updateSum, updateMax, updateCount = 0, 0, 0
    local planetClosest, cornered, arcs, arcOnDrive = math.huge, false, 0, 0
    local onArc, arcStart, finalAt = false, nil, nil
    -- The flight into a moving well: which phases the plan went through, and the ship at the moment
    -- the well took it.
    local directSeen, planSeen, wellEntry = false, false, nil
    -- How the drive runs through the entry: the closest the ship came to the entry corner, when the
    -- drive left `active`, and the largest angle between the nose and the velocity after the corner.
    local entryTrace = { closestToCorner = math.huge, maxNoseAngle = 0 }
    -- The nose at every step of a flight into a well, for the turn at its edge.
    local entrySamples = {}
    local shipWell = ship.entity:get(PhysicsComponents.GravityWell)
    for stepIndex = 1, math.floor(scenario.maxFlight / STEP) do
        local updateMs = self:step(ship, moving)
        if planet then
            planetClosest = math.min(planetClosest, ship.rb:getPos():distance(planet.planetBody.rb:getPos()))
        end
        local plan = ap:getPlan()
        local follower = plan and plan.follower
        if plan then
            directSeen = directSeen or plan.phase == PlanFlight.Phase.Direct
            planSeen = planSeen or plan.phase == PlanFlight.Phase.Plan
        end
        if scenario.outside and not wellEntry and shipWell:getBodyEntity() == well.planetBody.entity then
            wellEntry = {
                at = stepIndex * STEP,
                corner = follower and not follower.finished and follower.index or nil,
                speed = ShipFlightSystem:getSpeed(ship.entity),
                toPoint = follower and ship.rb:getPos():distance(follower.target) or nil,
                drive = drive:getState(),
                phase = plan and plan.phase,
                corners = follower and #follower.points or 0,
            }
        end
        -- The entry corner is passed once the follower has moved on from the corner it was on.
        if wellEntry and wellEntry.corner and not wellEntry.passedAt
            and (not follower or follower.finished or follower.index > wellEntry.corner) then
            wellEntry.passedAt = stepIndex * STEP
            wellEntry.passSpeed = ShipFlightSystem:getSpeed(ship.entity)
            wellEntry.passDrive = drive:getState()
        end
        if scenario.outside then
            local now = stepIndex * STEP
            if follower and not entryTrace.plannedGoal then
                entryTrace.plannedEntry, entryTrace.plannedGoal = follower.points[1].time, plan.goal.time
            end
            if wellEntry and wellEntry.corner and not wellEntry.passedAt and follower and not follower.finished then
                entryTrace.closestToCorner = math.min(entryTrace.closestToCorner,
                    ship.rb:getPos():distance(follower:pointNow(follower.points[wellEntry.corner])))
            end
            if driveSeen and not entryTrace.driveLeftAt and drive:getState() ~= "active" then
                entryTrace.driveLeftAt, entryTrace.driveLeftState = now, drive:getState()
                entryTrace.driveLeftSpeed, entryTrace.driveLeftPhase = ShipFlightSystem:getSpeed(ship.entity), plan and plan.phase
            end
            local velocity = ship.rb:getVelocity()
            local speed = velocity:length()
            if wellEntry and wellEntry.passedAt and speed > 1 then
                local nose = ship.rb:getRot():getForward()
                local cosine = (nose.x * velocity.x + nose.y * velocity.y + nose.z * velocity.z) / speed
                entryTrace.maxNoseAngle = math.max(entryTrace.maxNoseAngle, math.acos(math.max(-1, math.min(1, cosine))))
            end
            entrySamples[#entrySamples + 1] = { at = now, nose = ship.rb:getRot():getForward() }
        end
        cornered = cornered or follower ~= nil
        local arcNow = follower ~= nil and follower.arc ~= nil
        if plan and plan.phase == "final" and not finalAt then finalAt = stepIndex * STEP end
        if arcNow and not onArc then
            arcStart = stepIndex * STEP
            arcs = arcs + 1
            -- The drive is off for an arc: not charging or running (it may still be spinning down).
            if drive:getState() == "active" or drive:getState() == "charging" then arcOnDrive = arcOnDrive + 1 end
        end
        onArc = arcNow
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
    local endPlan = ap:getPlan()
    self:check(arrivedAt ~= nil, name .. "the autopilot arrives", string.format("%.0f s, %.0f from the target, phase %s, corner %s, drive %s",
        scenario.maxFlight, ship.rb:getPos():distance(aim()), endPlan and endPlan.phase or "-",
        endPlan and endPlan.follower and tostring(endPlan.follower.index) or "-", drive:getState())
        .. (endPlan and endPlan.follower and string.format(", %.1f from its point, speed %.2f, mult %.1f",
            ship.rb:getPos():distance(endPlan.follower.target), ShipFlightSystem:getSpeed(ship.entity),
            drive:getCurrentMult()) or ""))
    if not arrivedAt then
        self:despawn(ship, body, rocks)
        if moving then self:despawnWell(moving) end
        return
    end
    if etaAtDrive then
        local left = arrivedAt - driveAt
        self:check(math.abs(etaAtDrive - left) <= ETA_TOLERANCE * left, name .. "the ETA matches the flight",
            string.format("eta %.1f s, left %.1f s, drive at %.1f, arc at %s, final from %s, arrived %.1f", etaAtDrive, left,
                driveAt, tostring(arcStart), tostring(finalAt), arrivedAt))
    end

    local standOff = PlanFlight.standOff(targetRadius)
    local arrivalRadius = PlanFlight.arrivalRadius(targetRadius)
    local distance = ship.rb:getPos():distance(aim())
    self:check(math.abs(distance - standOff) <= arrivalRadius + hull, name .. "it arrives at the stand-off distance",
        string.format("%.1f against %.1f ± %.1f", distance, standOff, arrivalRadius))
    self:check(axesAreZero(input), name .. "disengaging leaves no axes")
    self:check(driveSeen == scenario.drive, name .. "the travel drive engages only when it should",
        string.format("drive seen %s", tostring(driveSeen)))

    -- It stops outside the hull, once the drive and the drag have run out.
    for _ = 1, math.floor(MAX_SETTLE_S / STEP) do
        if ShipFlightSystem:getSpeed(ship.entity) < STOPPED and drive:getState() == "idle" then break end
        self:step(ship, moving)
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

    if planet then
        self:check(planetClosest > PLANET_RADIUS + hull, name .. "the ship stays outside the planet's hull",
            string.format("closest %.1f, hull %.1f", planetClosest, PLANET_RADIUS + hull))
        self:check(cornered, name .. "the plan has a corner")
        self:check(arcOnDrive == 0, name .. "the drive is off when an arc starts",
            string.format("%d of %d arcs", arcOnDrive, arcs))
        Log.Info("ConstructsAutoPilotTest: %sclosest to the planet %.0f (hull %.0f, safe sphere %.0f), %d arcs, arc at %s s, final from %s s",
            name, planetClosest, PLANET_RADIUS, PLANET_RADIUS * PLANET_SAFE_FACTOR, arcs,
            tostring(arcStart), tostring(finalAt))
    end

    if scenario.outside then
        self:check(planSeen and not directSeen, name .. "the plan is flown, never replaced by a direct flight",
            string.format("plan seen %s, direct seen %s", tostring(planSeen), tostring(directSeen)))
        self:check(wellEntry ~= nil, name .. "the ship enters the planet's well")
        if wellEntry then
            -- The plan is the entry corner alone, and the well takes the ship before it passes it.
            self:check(wellEntry.corners == 1 and wellEntry.corner == 1,
                name .. "the well carries the ship before the entry corner is passed",
                string.format("%d corners, on corner %s", wellEntry.corners, tostring(wellEntry.corner)))
            self:check(wellEntry.passedAt ~= nil, name .. "the entry corner is passed")
            -- The drive run goes on through the entry: the drive is on when the corner is passed, and
            -- stays on until the ship arrives, where the autopilot lets it go.
            self:check(wellEntry.passDrive == "active", name .. "the drive is on at the entry corner",
                tostring(wellEntry.passDrive))
            self:check(entryTrace.driveLeftAt ~= nil and entryTrace.driveLeftAt >= arrivedAt - STEP / 2,
                name .. "the drive stays on until the ship arrives",
                string.format("left `active` at %s s (%s), arrived at %.1f s", tostring(entryTrace.driveLeftAt),
                    tostring(entryTrace.driveLeftState), arrivedAt))
            Log.Info("ConstructsAutoPilotTest: %sentered the well at %.1f s: phase %s, corner %s, speed %.2f, %s from its point, drive %s; corner passed at %s s at speed %s",
                name, wellEntry.at, tostring(wellEntry.phase), tostring(wellEntry.corner), wellEntry.speed,
                wellEntry.toPoint and string.format("%.1f", wellEntry.toPoint) or "-", wellEntry.drive,
                wellEntry.passedAt and string.format("%.1f", wellEntry.passedAt) or "-",
                wellEntry.passSpeed and string.format("%.2f", wellEntry.passSpeed) or "-")
        end
        if wellEntry then
            local swing = noseExcursion(samplesAround(entrySamples, wellEntry.at))
            self:check(swing <= MAX_EDGE_SWING, name .. "the nose does not swing at the well's edge",
                string.format("%.3f rad against %.2f, within %d s of the edge", swing, MAX_EDGE_SWING, EDGE_WINDOW_S))
            Log.Info("ConstructsAutoPilotTest: %sthe nose swings out and back by %.3f rad within %d s of the edge",
                name, swing, EDGE_WINDOW_S)
        end
        Log.Info("ConstructsAutoPilotTest: %sthe plan has the entry at %s s and the goal at %s s",
            name, tostring(entryTrace.plannedEntry), tostring(entryTrace.plannedGoal))
        Log.Info("ConstructsAutoPilotTest: %sthe drive started at %s s and left `active` at %s s (%s, phase %s, speed %s); closest to the entry corner %.2f, largest nose-to-velocity angle after it %.2f rad",
            name, tostring(driveAt), tostring(entryTrace.driveLeftAt), tostring(entryTrace.driveLeftState),
            tostring(entryTrace.driveLeftPhase), tostring(entryTrace.driveLeftSpeed), entryTrace.closestToCorner,
            entryTrace.maxNoseAngle)
    end

    self:despawn(ship, body, rocks)
    if moving then self:despawnWell(moving) end
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
