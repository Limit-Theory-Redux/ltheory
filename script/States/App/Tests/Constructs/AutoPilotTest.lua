-- Checks that the player's autopilot, a MoveTo order given through `AutoPilotSystem`, flies a ship
-- through its control input: it arrives at a point and at a body, with and without the travel
-- drive, stops outside the body's hull, steers around a rock on its course, and gives the controls
-- back to the player, with no axes left behind, and the order released.
-- They also arrive at a planet whose orbit is faster than the ship's top speed, from inside the
-- planet's gravity well, which carries the ship with the planet, and from outside it, where the travel
-- drive runs on through the well's entry to the stand-off point. A drive flight among rocks logs the
-- time the update takes with and without sensing, and each scenario logs its arrival time. Taking
-- the controls by hand cancels the order, and a new engagement while one is on gives it a new target.
-- Logs each failed check and ends with a failure exit code.

local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local AutoPilotSystem       = require("Modules.Constructs.Systems.AutoPilotSystem")
local ControlOwnerSystem    = require("Modules.Constructs.Systems.ControlOwnerSystem")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local Nav                   = require("Modules.Navigation")
local PlanFlight            = require("Shared.Helpers.PlanFlight")
local OrderGoals            = require("Shared.Helpers.OrderGoals")
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
-- A generated moon's hull radius, and how far a flight to one runs in the scenario: under 100 of the
-- moon's stand-offs (204), so the whole flight is shorter than the minimum a goal-sized rule gave.
local MOON_RADIUS   = 136
local MOON_DISTANCE = 12000
-- The flight round a moon: the moon lies this share of the way along the course, which is this
-- long, and its centre is this far off the course's line (along `MOON_ASIDE`, perpendicular to the
-- course). The course enters the moon's safe sphere only a little, so the detour round it is a
-- gentle corner: the drive turn cuts the corner towards the moon by a distance that grows with the
-- run's speed and the turn, and must stay within the detour's margin (a tenth of the safe radius).
-- The legs on either side are long enough for the turn at the speed the run has there (the turn
-- needs about 3.6 times the speed on each side of the corner), and the goal is far enough past it
-- that the braking law does not slow the run before the turn.
local MOON_COURSE_DISTANCE = 8000
local MOON_ALONG           = 0.3
local MOON_OFFSET          = 140
local MOON_ASIDE           = { x = 0.74 / 0.9528, y = 0, z = -0.6 / 0.9528 }
-- The turn's distance from the sharp path is measured while the ship is within this many leads
-- (`speed / drag`) of the corner.
local TURN_WINDOW_LEADS = 3
-- What the planned turn of that flight may be, in radians: gentle, but a real turn.
local MOON_TURN_MIN = math.rad(1.5)
local MOON_TURN_MAX = math.rad(40)
-- The planner keeps the ship this many hull radii from a planet; the ship is only checked to stay
-- outside the hull, and the clearance from this sphere is logged.
local PLANET_SAFE_FACTOR = Config.game.autoPilot.safeRadiusFactor.planet
local MOON_SAFE_FACTOR   = Config.game.autoPilot.safeRadiusFactor.moon
-- The wall-clock bound on waiting for a plan, in seconds: it turns a plan that never comes into a failure.
local PLAN_WAIT_S        = 30
-- The handover runs: how far their points lie from the start, how long the ship flies before the
-- player takes over or the target changes, and the bound on a flight to the new target. All in
-- game units and seconds.
local HANDOVER_DISTANCE     = 300
local HANDOVER_FLIGHT_S     = 2
local RETARGET_MAX_FLIGHT_S = 700
local MS_PER_S              = 1000

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
---@field maxFlight number Seconds of simulated time a flight may take to arrive: about 1.2 times what it takes
---@field rocks AutoPilotRock[]|nil Fixed spheres in the ship's way or beside it
---@field sensing boolean|nil False turns the autopilot's sensing off; the run is the baseline for its cost
---@field timed boolean|nil Whether the run's update times are logged, as a pair with `sensing` on and off
---@field well boolean|nil The target is a planet in a moving gravity well, and the ship starts inside the well
---@field planet boolean|nil A planet with a gravity well lies across the course, on a slow orbit
---@field moon boolean|nil With `planet`: the body across the course is a moon of `MOON_RADIUS`, not a planet
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
    { name = "a point, no drive",  distance = 300,   drive = false, maxFlight = 400 },
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
    -- Targets whose stand-off is large: the plan's legs are short against it, so a drive minimum that
    -- grew with the target's size would keep every leg in normal flight.
    { name = "a moon-sized body, with drive, past a planet", distance = MOON_DISTANCE, radius = MOON_RADIUS,
      drive = true, maxFlight = 60, planet = true },
    -- A gentle corner round a moon: the drive's run goes through it, with one charge. A flight that
    -- releases the drive at the corner takes about 60 s, and this one about 42 s.
    { name = "a point, with drive, round a moon", distance = MOON_COURSE_DISTANCE, drive = true, maxFlight = 50,
      planet = true, moon = true },
    -- Far enough that the drive's charge and ramp-up, which the ETA doesn't count, are a small part of the flight.
    { name = "a planet-sized body, with drive", distance = 60000, radius = PLANET_RADIUS, drive = true,
      maxFlight = 32 },
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
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("ConstructsAutoPilotTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---What flies the ship in a run, and the calls the checks read it through.
---@class AutoPilotFlyer
---@field name string
---@field engage fun(target: Entity|nil, targetPos: Position) Starts the flight to the entity, or to the point when there is none; while a flight is on, gives it the new target
---@field stopAt fun(): gameunit, gameunit The distance from the target's centre the flight stops at (0 for a point), and how far from there counts as arrived
---@field sensingOff fun() Turns the sensing of the flight's navigation world off
---@field flying fun(): boolean Whether the flight is still on
---@field arrived fun(): boolean Whether a flight that has ended ended at its goal
---@field handedBack fun(): boolean Whether the player has the controls again and the order is released
---@field planning fun(): boolean Whether the plan has not come yet
---@field update fun(dt: number) The flight's update, which writes the ship's control input
---@field plan fun(): PlanFlightState|nil
---@field eta fun(): number Seconds the flight has left, by its own estimate
---@field finish fun() Ends the flight if it is on and lets go of what it holds

---The player's autopilot flies the ship: a MoveTo order issued through `AutoPilotSystem`, flown by
---the order system in a navigation world of its own.
---@param ship { entity: Entity }
---@param world Physics
---@return AutoPilotFlyer
local function playerFlyer(ship, world)
    Nav.attach(world)
    local Status = Enums.NavOrderStatus
    local ap = ship.entity:get(ConstructComponents.AutoPilot)
    ---@type Entity
    local order
    ---@type NavTarget
    local flownTo
    -- How the order ended, read before the autopilot system releases it.
    ---@type NavOrderStatusId|nil
    local endedAs
    return {
        name = "player autopilot",
        engage = function(target, targetPos)
            flownTo = target or targetPos
            AutoPilotSystem:engage(ship.entity, flownTo)
            order = ap:getOrder()
        end,
        stopAt = function()
            local goal = OrderGoals.moveTo(flownTo)
            return goal.safeRange, goal.arrivalRadius
        end,
        sensingOff = function()
            for _, navWorld in Registry:view(Nav.Components.NavWorld) do navWorld:getNav():setSensing(0) end
        end,
        flying = function() return AutoPilotSystem:isActive(ship.entity) end,
        arrived = function() return endedAs == Status.Done end,
        handedBack = function()
            return ControlOwnerSystem:ownerOf(ship.entity) == Enums.ControlOwner.Manual
                and not AutoPilotSystem:isActive(ship.entity) and #ap:getEnding() == 0 and not order:isValid()
        end,
        planning = function() return AutoPilotSystem:isPlanning(ship.entity) end,
        update = function(dt)
            Nav.update(dt)
            if not endedAs and order:get(Nav.Components.OrderStatus):hasEnded() then endedAs = Nav.status(order) end
            AutoPilotSystem:update()
        end,
        plan = function()
            local flight = order:isValid() and order:get(Nav.Components.OrderFlight)
            return flight and flight.plan or nil
        end,
        eta = function() return AutoPilotSystem:getProgress(ship.entity).eta end,
        finish = function()
            AutoPilotSystem:disengage(ship.entity)
            Nav.update(0)
            AutoPilotSystem:update()
            Nav.detach()
        end,
    }
end

---The ways a scenario is flown. `autoPilot`: the ship carries the autopilot component.
local FLYERS = {
    { name = "player autopilot", autoPilot = true, make = playerFlyer },
}

---A ship set up like the player ship, at `start`, with the autopilot component when `autoPilot`
---and, when `inWell`, a gravity well component that lets a zone carry it. The player pilots it:
---manual input owns its controls.
---@param start Position
---@param inWell boolean|nil
---@param autoPilot boolean
function AutoPilotTest:spawnShip(start, inWell, autoPilot)
    local ship = FlightScene.spawnShip(self.world, start, { inWell = inWell, autoPilot = autoPilot })
    ControlOwnerSystem:setManual(ship.entity, true)
    return ship
end

---A planet of `radius` at `planetPos` on a circular orbit around a star, with a gravity well, whose
---speed on its orbit is `speedup` times the Solo top speed.
---@param planetPos Position
---@param radius gameunit
---@param speedup number
---@param options { moon: boolean|nil }|nil As `FlightScene.buildWell`
---@return table well `planetBody` is the planet as a target: its entity and rigid body
function AutoPilotTest:buildWell(planetPos, radius, speedup, options)
    return FlightScene.buildWell(self.world, planetPos, radius, speedup, options)
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

---One simulation step in the order the apps use: the flight's update at PreSim, then the drive, the
---orbits and wells of a moving well, flight and physics.
---@param flyer AutoPilotFlyer
---@param well table|nil The scenario's moving well
---@return number updateMs Milliseconds the flight's update took
function AutoPilotTest:step(flyer, well)
    local start = InstantTime.Now()
    flyer.update(STEP)
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

---Wait for the plan of a flight that has just been engaged, from a worker. The ship is not stepped,
---and `update` with no time passing only polls. The first update takes up the engagement.
---@param flyer AutoPilotFlyer
---@return boolean planned False when the wait ran out
function AutoPilotTest:awaitPlan(flyer)
    flyer.update(0)
    local waitStart = InstantTime.Now()
    while flyer.planning() and InstantTime.Now():durationSince(waitStart) < PLAN_WAIT_S * MS_PER_S do
        flyer.update(0)
    end
    return not flyer.planning()
end

---The distance from `p` to the segment from `a` to `b`, all `{ x, y, z }`.
local function segmentDistance(p, a, b)
    local abx, aby, abz = b.x - a.x, b.y - a.y, b.z - a.z
    local length2 = abx * abx + aby * aby + abz * abz
    local along = length2 > 0 and ((p.x - a.x) * abx + (p.y - a.y) * aby + (p.z - a.z) * abz) / length2 or 0
    along = math.max(0, math.min(1, along))
    local dx, dy, dz = p.x - (a.x + abx * along), p.y - (a.y + aby * along), p.z - (a.z + abz * along)
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

---The turn of a plan of one corner, read when its follower first exists: the corner's turn angle
---and whether every corner keeps the drive run, and what the flight is then measured against.
---@param plan PlanFlightState
---@param follower PlanFollower|nil
---@return table|nil trace Nil while the plan has no follower
function AutoPilotTest:captureTurn(plan, follower)
    if not follower then return nil end
    local corner, goal, start = follower:pointNow(follower.points[1]), follower:pointNow(plan.goal), plan.start
    local through = true
    for _, point in ipairs(follower.points) do through = through and point.through == true and point.radius == 0 end
    return {
        start = start, corner = corner, goal = goal, corners = #follower.points, through = through,
        drag = follower.drag, maxOff = 0,
        turn = angleBetween({ x = corner.x - start.x, y = corner.y - start.y, z = corner.z - start.z },
            { x = goal.x - corner.x, y = goal.y - corner.y, z = goal.z - corner.z }),
    }
end

---One step of a turn's measure: the largest distance of the ship from the plan's sharp-cornered
---path, and the ship at the moment its follower has passed the corner (its speed, its drive and how
---far it still is from the corner).
function AutoPilotTest:traceTurn(trace, plan, follower, ship, driveState)
    if plan.phase ~= PlanFlight.Phase.Plan and plan.phase ~= PlanFlight.Phase.Final then return end
    local at = ship.rb:getPos()
    -- The corner is a point of the moon's frame, so it moves with the moon: it is where the plan puts
    -- it only at the time the ship reaches it, and the distance is taken only while the ship is near.
    local corner = follower and follower:pointNow(follower.points[1]) or trace.corner
    local lead = ShipFlightSystem:getSpeed(ship.entity) / trace.drag
    if at:distance(corner) <= TURN_WINDOW_LEADS * lead then
        trace.maxOff = math.max(trace.maxOff, math.min(segmentDistance(at, trace.start, corner),
            segmentDistance(at, corner, trace.goal)))
    end
    if not trace.passSpeed and follower and (follower.finished or follower.index > 1) then
        trace.passSpeed = ShipFlightSystem:getSpeed(ship.entity)
        trace.passDrive = driveState
        trace.passDistance = at:distance(corner)
    end
end

---The checks of the flight round a moon: its corner keeps the drive run, with one charge and no arc.
function AutoPilotTest:checkMoonTurn(name, trace, arcs, charges)
    self:check(trace ~= nil, name .. "the plan has a corner to turn at")
    if not trace then return end
    self:check(trace.through, name .. "every corner keeps the drive run (no arc)", string.format("%d corners", trace.corners))
    self:check(trace.turn >= MOON_TURN_MIN and trace.turn <= MOON_TURN_MAX, name .. "the corner is a gentle turn",
        string.format("%.1f deg", math.deg(trace.turn)))
    self:check(arcs == 0, name .. "no arc is flown", string.format("%d arcs", arcs))
    self:check(charges == 1, name .. "the drive charges once", string.format("%d charges", charges))
    self:check(trace.passDrive == "active", name .. "the drive is on at the corner", tostring(trace.passDrive))
    Log.Info("ConstructsAutoPilotTest: %sturn %.1f deg, %d corners; passed at speed %s with the drive %s, %s from the corner (model lead %s); "
        .. "largest distance from the sharp path %.1f; %d charges",
        name, math.deg(trace.turn), trace.corners, trace.passSpeed and string.format("%.1f", trace.passSpeed) or "-",
        tostring(trace.passDrive), trace.passDistance and string.format("%.1f", trace.passDistance) or "-",
        trace.passSpeed and string.format("%.1f", trace.passSpeed / trace.drag) or "-", trace.maxOff, charges)
end

---Fly one scenario and check it. A scenario that expects no drive flies with the autopilot's drive
---switched off: a leg longer than the drive's shortest (about a minute of top speed) is driven, and
---nothing avoids a rock under the drive, so a short flight past a rock is flown in normal flight.
---@param scenario AutoPilotScenario
---@param kind { name: string, autoPilot: boolean, make: fun(ship: table, world: Physics): AutoPilotFlyer } What flies it
---@return number|nil arrivedAt Seconds of simulated time the flight took; nil when it did not arrive
function AutoPilotTest:run(scenario, kind)
    local autoPilot = Config.game.autoPilot
    local driveEnabled = autoPilot.enableTravelDrive
    if not scenario.drive then autoPilot.enableTravelDrive = false end
    local arrivedAt = self:fly(scenario, kind)
    autoPilot.enableTravelDrive = driveEnabled
    return arrivedAt
end

---The flight of `run`, with the drive's setting as the scenario wants it.
---@param scenario AutoPilotScenario
---@param kind { name: string, autoPilot: boolean, make: fun(ship: table, world: Physics): AutoPilotFlyer }
---@return number|nil arrivedAt
function AutoPilotTest:fly(scenario, kind)
    local name = kind.name .. ", " .. scenario.name .. ": "
    local length = math.sqrt(DIRECTION.x ^ 2 + DIRECTION.y ^ 2 + DIRECTION.z ^ 2)
    local well = scenario.well and self:buildWell(Position(WELL_ORBIT_RADIUS, 0, 0), BODY_RADIUS,
        scenario.wellSpeedup or WELL_ORBIT_SPEEDUP)
    -- The planet across the course: the ship starts at the origin, so the course runs from there.
    -- A moon, when the scenario has one, is the body across the course instead of a planet.
    local bodyRadius = scenario.moon and MOON_RADIUS or PLANET_RADIUS
    local bodySafeFactor = scenario.moon and MOON_SAFE_FACTOR or PLANET_SAFE_FACTOR
    local along = scenario.moon and MOON_ALONG or PLANET_ALONG
    local aside = scenario.moon and MOON_OFFSET or 0
    local planet = scenario.planet and self:buildWell(Position(
        DIRECTION.x / length * scenario.distance * along + MOON_ASIDE.x * aside,
        DIRECTION.y / length * scenario.distance * along + MOON_ASIDE.y * aside,
        DIRECTION.z / length * scenario.distance * along + MOON_ASIDE.z * aside), bodyRadius, PLANET_ORBIT_SPEEDUP,
        { moon = scenario.moon })
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

    local ship = self:spawnShip(origin, well ~= nil, kind.autoPilot)
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    local flyer = kind.make(ship, self.world)
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
    flyer.engage(body and body.entity, targetPos)
    if scenario.sensing == false then
        flyer.sensingOff()
    end
    local targetRadius = scenario.radius or 0
    local hull = ship.rb:getBoundingRadius()
    self:check(flyer.flying(), name .. "engaging starts the flight")

    -- The plan comes from a worker: wait for it, so that every run flies the same way.
    self:check(self:awaitPlan(flyer), name .. "the plan comes")

    -- The first update writes the axes of a ship that is far from its goal and not facing it.
    flyer.update(STEP)
    self:check(not axesAreZero(input), name .. "the flight writes the control input")

    local closest, driveSeen, arrivedAt = math.huge, false, nil
    local rockClosest = math.huge
    local etaAtDrive, driveAt
    local updateSum, updateMax, updateCount = 0, 0, 0
    local planetClosest, cornered, arcs, arcOnDrive = math.huge, false, 0, 0
    -- The turn of a flight round a moon, and how many times the drive began to charge.
    local turnTrace, charges, lastDriveState = nil, 0, nil
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
        local updateMs = self:step(flyer, moving)
        if planet then
            planetClosest = math.min(planetClosest, ship.rb:getPos():distance(planet.planetBody.rb:getPos()))
        end
        local plan = flyer.plan()
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
        local driveState = drive:getState()
        if driveState == "charging" and lastDriveState ~= "charging" then charges = charges + 1 end
        lastDriveState = driveState
        if scenario.moon and plan then
            turnTrace = turnTrace or self:captureTurn(plan, follower)
            if turnTrace then self:traceTurn(turnTrace, plan, follower, ship, driveState) end
        end
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
            driveSeen, etaAtDrive, driveAt = true, flyer.eta(), stepIndex * STEP
        end
        if not flyer.flying() then
            arrivedAt = stepIndex * STEP
            break
        end
    end
    local endPlan = flyer.plan()
    local arrived = arrivedAt ~= nil and flyer.arrived()
    self:check(arrived, name .. "it arrives", string.format("%.0f s, %.0f from the target, phase %s, corner %s, drive %s",
        scenario.maxFlight, ship.rb:getPos():distance(aim()), endPlan and endPlan.phase or "-",
        endPlan and endPlan.follower and tostring(endPlan.follower.index) or "-", drive:getState())
        .. (endPlan and endPlan.follower and string.format(", %.1f from its point, speed %.2f, mult %.1f",
            ship.rb:getPos():distance(endPlan.follower.target), ShipFlightSystem:getSpeed(ship.entity),
            drive:getCurrentMult()) or ""))
    if not arrived then
        flyer.finish()
        self:despawn(ship, body, rocks)
        if moving then self:despawnWell(moving) end
        return nil
    end
    if etaAtDrive then
        local left = arrivedAt - driveAt
        self:check(math.abs(etaAtDrive - left) <= ETA_TOLERANCE * left, name .. "the ETA matches the flight",
            string.format("eta %.1f s, left %.1f s, drive at %.1f, arc at %s, final from %s, arrived %.1f", etaAtDrive, left,
                driveAt, tostring(arcStart), tostring(finalAt), arrivedAt))
    end

    local standOff, arrivalRadius = flyer.stopAt()
    -- A flight that stops short of its target stays outside the target's hull; one that flies to a
    -- point has no hull to stay outside of.
    local clearance = standOff > 0 and targetRadius + hull or 0
    local distance = ship.rb:getPos():distance(aim())
    self:check(math.abs(distance - standOff) <= arrivalRadius + hull, name .. "it arrives at the stand-off distance",
        string.format("%.1f against %.1f ± %.1f", distance, standOff, arrivalRadius))
    self:check(axesAreZero(input), name .. "disengaging leaves no axes")
    self:check(flyer.handedBack(), name .. "the player has the controls again and the order is released")
    self:check(driveSeen == scenario.drive, name .. "the travel drive engages only when it should",
        string.format("drive seen %s", tostring(driveSeen)))

    -- It stops outside the hull, once the drive and the drag have run out.
    for _ = 1, math.floor(MAX_SETTLE_S / STEP) do
        if ShipFlightSystem:getSpeed(ship.entity) < STOPPED and drive:getState() == "idle" then break end
        self:step(flyer, moving)
        closest = math.min(closest, ship.rb:getPos():distance(aim()))
        rockClosest = math.min(rockClosest, self:closestRockGap(ship, rocks))
    end
    -- At rest relative to the well that carries it (its world speed in open space).
    local restSpeed = ShipFlightSystem:getSpeed(ship.entity)
    self:check(restSpeed < STOPPED, name .. "the ship comes to rest", tostring(restSpeed))
    self:check(drive:getState() == "idle", name .. "the drive is idle", drive:getState())
    self:check(closest > clearance, name .. "the ship stays outside the target's hull",
        string.format("closest %.1f, hull %.1f", closest, clearance))
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
        self.updateMeans[kind.name] = self.updateMeans[kind.name] or {}
        self.updateMeans[kind.name][scenario.sensing == false and "off" or "on"] = mean
        Log.Info("ConstructsAutoPilotTest: %supdate per frame: mean %.4f ms, max %.4f ms over %d frames",
            name, mean, updateMax, updateCount)
    end

    if planet then
        self:check(planetClosest > bodyRadius + hull, name .. "the ship stays outside the planet's hull",
            string.format("closest %.1f, hull %.1f", planetClosest, bodyRadius + hull))
        self:check(cornered, name .. "the plan has a corner")
        self:check(arcOnDrive == 0, name .. "the drive is off when an arc starts",
            string.format("%d of %d arcs", arcOnDrive, arcs))
        Log.Info("ConstructsAutoPilotTest: %sclosest to the planet %.0f (hull %.0f, safe sphere %.0f), %d arcs, arc at %s s, final from %s s",
            name, planetClosest, bodyRadius, bodyRadius * bodySafeFactor, arcs,
            tostring(arcStart), tostring(finalAt))
    end

    if scenario.moon then
        self:checkMoonTurn(name, turnTrace, arcs, charges)
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

    flyer.finish()
    self:despawn(ship, body, rocks)
    if moving then self:despawnWell(moving) end
    return arrivedAt
end

---Where the handover runs fly: a point this far from the start, along `DIRECTION` or against it.
---@param sign number 1 or -1
---@return Position
local function handoverPoint(sign)
    local length = math.sqrt(DIRECTION.x ^ 2 + DIRECTION.y ^ 2 + DIRECTION.z ^ 2)
    local reach = sign * HANDOVER_DISTANCE / length
    return Position(DIRECTION.x * reach, DIRECTION.y * reach, DIRECTION.z * reach)
end

---The player takes the controls during a flight: the order is cancelled and released, the player
---has the controls at once with no axes left from the autopilot, and engaging again starts a flight.
function AutoPilotTest:runTakeover()
    local name = "takeover: "
    local ship = self:spawnShip(Position(0, 0, 0), false, true)
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local flyer = playerFlyer(ship, self.world)
    self:check(AutoPilotSystem:getRoute(ship.entity) == nil, name .. "there is no route before the autopilot is engaged")
    flyer.engage(nil, handoverPoint(1))
    self:check(ControlOwnerSystem:ownerOf(ship.entity) == Enums.ControlOwner.Orders,
        name .. "the order owns the ship's controls while it flies")
    self:check(self:awaitPlan(flyer), name .. "the plan comes")
    for _ = 1, math.floor(HANDOVER_FLIGHT_S / STEP) do self:step(flyer) end
    self:check(not axesAreZero(input), name .. "the flight writes the control input")
    local route = AutoPilotSystem:getRoute(ship.entity)
    self:check(route ~= nil and #route >= 1, name .. "the autopilot has a route while it flies")
    self:check(route ~= nil and route[#route]:distance(AutoPilotSystem:getTargetPosition(ship.entity)) < STOPPED,
        name .. "the route ends at the target (a bare point has no stand-off)")

    AutoPilotSystem:disengage(ship.entity)
    self:check(AutoPilotSystem:getRoute(ship.entity) == nil, name .. "there is no route once it is disengaged")
    self:check(not flyer.flying(), name .. "disengaging ends the flight")
    self:check(ControlOwnerSystem:ownerOf(ship.entity) == Enums.ControlOwner.Manual,
        name .. "the player owns the controls at once")
    self:check(axesAreZero(input), name .. "the autopilot's axes are cleared")
    -- The order system ends the cancelled order at its next update; the autopilot system releases it.
    self:step(flyer)
    self:check(flyer.handedBack(), name .. "the cancelled order is released")
    for _ = 1, math.floor(HANDOVER_FLIGHT_S / STEP) do self:step(flyer) end
    self:check(axesAreZero(input), name .. "nothing writes the input once the player has the controls")

    flyer.engage(nil, handoverPoint(-1))
    self:check(flyer.flying(), name .. "engaging again starts a flight")
    self:check(ControlOwnerSystem:ownerOf(ship.entity) == Enums.ControlOwner.Orders,
        name .. "the new order owns the controls")
    flyer.finish()
    self:despawn(ship, nil, {})
end

---Engaging while the autopilot flies gives its order a new target, and the ship flies to it.
function AutoPilotTest:runRetarget()
    local name = "retarget: "
    local ship = self:spawnShip(Position(0, 0, 0), false, true)
    local ap = ship.entity:get(ConstructComponents.AutoPilot)
    local flyer = playerFlyer(ship, self.world)
    flyer.engage(nil, handoverPoint(1))
    local order = ap:getOrder()
    self:check(self:awaitPlan(flyer), name .. "the first plan comes")
    for _ = 1, math.floor(HANDOVER_FLIGHT_S / STEP) do self:step(flyer) end

    local newTarget = handoverPoint(-1)
    flyer.engage(nil, newTarget)
    self:check(ap:getOrder() == order, name .. "the flight keeps its order")
    local goal = AutoPilotSystem:getTargetPosition(ship.entity)
    self:check(goal ~= nil and goal:distance(newTarget) < STOPPED, name .. "the order has the new target")
    self:check(self:awaitPlan(flyer), name .. "the new plan comes")
    local flownFor = 0
    while flyer.flying() and flownFor < RETARGET_MAX_FLIGHT_S do
        self:step(flyer)
        flownFor = flownFor + STEP
    end
    local _, arrivalRadius = flyer.stopAt()
    self:check(flyer.arrived(), name .. "it arrives", string.format("%.0f s", flownFor))
    self:check(ship.rb:getPos():distance(newTarget) <= arrivalRadius + ship.rb:getBoundingRadius(),
        name .. "it arrives at the new target")
    flyer.finish()
    self:despawn(ship, nil, {})
end

function AutoPilotTest:onInit()
    self.passed, self.failed = 0, 0
    -- Mean update time of the timed runs, in milliseconds, by what flew and whether sensing was on.
    self.updateMeans = {}

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    for _, scenario in ipairs(SCENARIOS) do
        local times = {}
        for _, kind in ipairs(FLYERS) do
            local arrivedAt = self:run(scenario, kind)
            times[#times + 1] = string.format("%s %s", kind.name, arrivedAt and string.format("%.1f s", arrivedAt) or "none")
        end
        Log.Info("ConstructsAutoPilotTest: %s: arrival after %s", scenario.name, table.concat(times, ", "))
    end

    for _, kind in ipairs(FLYERS) do
        local means = self.updateMeans[kind.name]
        if means and means.on and means.off then
            Log.Info("ConstructsAutoPilotTest: sensing costs %.4f ms per frame in the %s update (%.4f on, %.4f off)",
                means.on - means.off, kind.name, means.on, means.off)
        end
    end

    self:runTakeover()
    self:runRetarget()

    Log.Info("ConstructsAutoPilotTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return AutoPilotTest
