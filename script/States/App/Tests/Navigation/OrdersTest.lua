-- Checks the navigation orders: MoveTo to a point past a planet (a plan with a corner, the drive) and
-- to a body, the approach specs (stand-off, point, refused), the facing goal, a queue, priorities, control ownership (a claim, a piloted ship), Patrol with and without a loop, Hold, cancel (a queued order and a running
-- one), a change of target in flight (the path is planned again), of cost profile (a new plan replaces the
-- old one while the ship flies on), a target and
-- a ship that are destroyed, Orbit (a named plane, the ship's plane, a planet in a moving well), Follow
-- (in and out of the join range, a leader that leaves it), the refused specs, release, and the world's detach. The end events are checked in frames
-- after the flights, since the event bus delivers them there. Logs each failed check and ends
-- with a failure exit code.

local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local ConstructComponents   = require("Modules.Constructs.Components")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local ControlOwnerSystem    = require("Modules.Constructs.Systems.ControlOwnerSystem")
local Nav                   = require("Modules.Navigation")
local NavConfig             = require("Config.Navigation.NavConfig")
local CostProfileDefs       = require("Shared.Registries.CostProfileRegistry")
local PlanFlight            = require("Shared.Helpers.PlanFlight")
local ApproachSpec          = require("Shared.Types.ApproachSpec")
local ApproachSpecs         = require("Shared.Helpers.ApproachSpecs")
local FlightScene           = require("States.App.Tests.Support.FlightScene")
local ScaleConfig           = require("Config.Gen.UniverseScaleConfig")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")

---@class NavigationOrdersTest: Application
local OrdersTest = Subclass("NavigationOrdersTest", Application)

local Status     = Enums.NavOrderStatus
local Failure    = Enums.NavOrderFailure
local Owner      = Enums.ControlOwner
local Phase      = PlanFlight.Phase
local Components = Nav.Components

local STEP = 1 / 60
-- Bounds that turn a flight or a wait that never ends into a failure.
local MAX_FLIGHT_S = 240
local PLAN_WAIT_S  = 30
local MS_PER_S     = 1000
local MAX_FRAMES   = 600
-- Frames the events get to arrive in after the last flight.
local EVENT_FRAMES = 3

-- Distances, in game units. A point goal within the drive's minimum length flies without the drive.
local LEG        = 40
local BODY_RADIUS = 1
-- The gap from the body's hull a stop is ordered at, in metres.
local GAP_M      = 100000
local SECONDS_BEFORE_CHANGE = 10
-- A ship that has arrived is within this many arrival radii of its goal.
local ARRIVED_SLACK = 2
-- A ship counts as moving above this speed, in game units per second.
local MOVING = 0.5
local STOPPED = 0.05
-- Past a planet: the course's length and where the planet lies across it, and the planet itself.
local PLANET_COURSE = { x = 12000, y = 0, z = 9000 }
local PLANET_ALONG  = 0.5
local PLANET_RADIUS = 500
local PLANET_ORBIT_SPEEDUP = 0.01
local PLANET_COURSE_S = 140
-- A body far enough that the default stand-off is a flight (its distance in legs and its radius), and
-- the radii an inline spec asks for, which stop the ship farther out than the default.
local FAR_BODY_LEGS   = 3.5
local FAR_BODY_RADIUS = 25
local WIDE_RADII      = 3
-- A station: its radius, so that its approach point is clear of the hull.
local STATION_RADIUS = 10
-- A facing holds within this angle of the point, in radians; the flight a facing is checked on, in game units.
local FACING_TOLERANCE = 0.2
local FACING_FLIGHT    = LEG

function OrdersTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationOrdersTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---@param input ControlInputComponent
local function axesAreZero(input)
    return input:getThrustForward() == 0 and input:getThrustRight() == 0 and input:getThrustUp() == 0
        and input:getPitch() == 0 and input:getYaw() == 0 and input:getRoll() == 0
end

---@return NavWorldComponent
local function worldComponent()
    for _, world in Registry:view(Components.NavWorld) do return world end
    error("no navigation world")
end

---@param ship { entity: Entity }
---@return boolean
local function hasAgent(ship)
    return ship.entity:get(Components.NavAgent) ~= nil
end

---@param order Entity
---@return PlanFlightState|nil
local function planOf(order)
    return order:get(Components.OrderFlight).plan
end

---One step of the simulation, in the order a frame runs it.
---@param well table|nil The moving well of the scene
function OrdersTest:step(well)
    Nav.update(STEP)
    TravelDriveSystem:update(STEP)
    if well then
        OrbitalSystem:update(well.orbiters, STEP, well.followers)
        GravityWellSystem:update(STEP, self.world:getSolverSubsteps())
    end
    ShipFlightSystem:update()
    self.world:update(STEP)
end

---Steps until `done()` holds, for at most `seconds` of simulated time.
---@param done fun(): boolean
---@param seconds number
---@param well table|nil
---@return boolean reached
function OrdersTest:flyUntil(done, seconds, well)
    for _ = 1, math.floor(seconds / STEP) do
        if done() then return true end
        self:step(well)
    end
    return done()
end

---Updates with no time passing until the order's plan has come back; the ship doesn't move.
---@param order Entity
function OrdersTest:awaitPlan(order)
    local start = InstantTime.Now()
    while Nav.status(order) == Status.Planning
        and InstantTime.Now():durationSince(start) < PLAN_WAIT_S * MS_PER_S do
        Nav.update(0)
    end
end

---Remembers the event an ended order is to be announced with.
---@param order Entity
function OrdersTest:expectEnd(order)
    local status = Nav.status(order)
    self:check(status == Status.Done or status == Status.Failed or status == Status.Cancelled,
        "an order that is checked for its event has ended")
    self.expected[order.id] = status
end

---Ends a case: releases its orders (they have ended), and removes its ships and bodies.
---@param orders Entity[]
---@param things { entity: Entity, rb: RigidBody }[]
function OrdersTest:finish(orders, things)
    for _, order in ipairs(orders) do
        self:expectEnd(order)
        Nav.release(order)
        self:check(Registry:getEntityById(order.id) == nil, "a released order is gone")
    end
    for _, thing in ipairs(things) do
        if thing.entity:isValid() then
            self.world:removeRigidBody(thing.rb)
            Registry:destroyEntity(thing.entity)
        end
    end
    -- The query index of the next case starts without them.
    self.world:update(STEP)
end

---@param at Position
---@return { entity: Entity, rb: RigidBody }
function OrdersTest:ship(at)
    local ship = FlightScene.spawnShip(self.world, at)
    self.world:update(STEP)
    return ship
end

---@param ship { rb: RigidBody }
---@param goal Position
---@param radius gameunit
---@return boolean
local function within(ship, goal, radius)
    return ship.rb:getPos():distance(goal) <= radius
end

function OrdersTest:caseMoveToPastPlanet()
    local name = "a point past a planet: "
    local course = Position(PLANET_COURSE.x, PLANET_COURSE.y, PLANET_COURSE.z)
    local planet = FlightScene.buildWell(self.world,
        Position(course.x * PLANET_ALONG, course.y * PLANET_ALONG, course.z * PLANET_ALONG),
        PLANET_RADIUS, PLANET_ORBIT_SPEEDUP)
    local ship = self:ship(Position(0, 0, 0))
    self.world:update(STEP)

    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = course })
    self:check(Nav.status(order) == Status.Queued, name .. "a new order is queued")
    Nav.update(0)
    self:check(Nav.status(order) == Status.Planning, name .. "the order plans")
    self:check(hasAgent(ship), name .. "a ship with an order has an agent")
    self:awaitPlan(order)
    self:check(Nav.status(order) == Status.Active, name .. "the plan comes and the order is active")
    local plan = planOf(order)
    self:check(plan and plan.phase == Phase.Plan, name .. "the plan has corners")

    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    local driveSeen, closest, progressSeen = false, math.huge, false
    local reached = self:flyUntil(function()
        if drive:getState() ~= "idle" then driveSeen = true end
        closest = math.min(closest, ship.rb:getPos():distance(planet.planetPos))
        local progress = Nav.progress(order)
        if progress.distanceLeft and progress.eta and progress.distanceLeft > 0 and progress.eta > 0 then
            progressSeen = true
        end
        return Nav.status(order) == Status.Done
    end, PLANET_COURSE_S, planet)
    self:check(reached, name .. "the order is done")
    self:check(driveSeen, name .. "the travel drive runs")
    self:check(progressSeen, name .. "progress has a distance and an ETA")
    self:check(closest > PLANET_RADIUS + ship.rb:getBoundingRadius(), name .. "the ship stays outside the planet",
        string.format("closest %.1f", closest))
    self:check(within(ship, course, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the point")
    self:check(not hasAgent(ship), name .. "the agent is gone with the last order")
    self:check(axesAreZero(ship.entity:get(ConstructComponents.ControlInput)), name .. "no axes are left")
    FlightScene.despawnWell(self.world, planet)
    FlightScene.despawn(self.world, { planet.planetBody })
    self:finish({ order }, { ship })
end

function OrdersTest:caseMoveToBody()
    local name = "a body: "
    local ship = self:ship(Position(0, 0, 0))
    local bodyPos = Position(LEG, LEG / 4, -LEG / 2)
    local body = FlightScene.spawnBody(self.world, bodyPos, BODY_RADIUS)
    self.world:update(STEP)

    local approach = ApproachSpec.Of { kind = Enums.ApproachKind.StandOff, gapM = GAP_M }
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity, approach = approach })
    local gap = approach:getGap()
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    local dist = ship.rb:getPos():distance(bodyPos)
    local hull = ship.rb:getBoundingRadius()
    local arrival = PlanFlight.arrivalRadius(BODY_RADIUS, gap)
    self:check(dist >= BODY_RADIUS + hull, name .. "the ship is outside the body's hull", string.format("%.3f", dist))
    self:check(math.abs(dist - PlanFlight.standOff(BODY_RADIUS, gap)) <= arrival * ARRIVED_SLACK,
        name .. "the ship stops at the stand-off", string.format("%.3f of %.3f", dist,
            PlanFlight.standOff(BODY_RADIUS, gap)))
    self:finish({ order }, { ship, body })
end

---Flies a MoveTo to a large body 640 game units away, and returns how far from its centre the ship stops.
---@param approach ApproachSpecId|ApproachSpecConstructor|nil
---@param markers Component[]|nil Added to the body, to make it a kind of target
---@return number dist
---@return gameunit arrival The arrival radius of the stand-off
function OrdersTest:standOffTo(approach, markers)
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG * FAR_BODY_LEGS, 0, 0), FAR_BODY_RADIUS)
    for _, marker in ipairs(markers or {}) do body.entity:add(marker) end
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity, approach = approach })
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    local dist = ship.rb:getPos():distance(body.rb:getPos())
    self:check(reached, "an approach to a body: the order is done",
        string.format("status %s, %.1f from the body", tostring(Nav.status(order)), dist))
    if not reached then Nav.cancel(order); self:step() end
    self:finish({ order }, { ship, body })
    local radii = approach and ApproachSpec.Of(approach):getRadii() or NavConfig.standOffRadii
    return dist, PlanFlight.arrivalRadius(FAR_BODY_RADIUS, nil, radii)
end

function OrdersTest:caseApproachStandOff()
    local name = "a stand-off approach: "
    local default = ApproachSpecs.For(Registry:createEntity(), nil)
    self:check(not default:isPoint() and default:getRadii() == NavConfig.standOffRadii,
        name .. "a target of no kind gets the generic stand-off")

    local dist, arrival = self:standOffTo(nil)
    local want = PlanFlight.standOff(FAR_BODY_RADIUS, nil, NavConfig.standOffRadii)
    self:check(math.abs(dist - want) <= arrival * ARRIVED_SLACK, name .. "without an approach, the default stand-off",
        string.format("%.1f of %.1f", dist, want))

    dist, arrival = self:standOffTo(Enums.ApproachSpec.PlanetStandOff)
    self:check(math.abs(dist - want) <= arrival * ARRIVED_SLACK, name .. "a named spec stops at its stand-off",
        string.format("%.1f of %.1f", dist, want))

    local wide = { kind = Enums.ApproachKind.StandOff, radii = WIDE_RADII }
    local defaultStop = dist
    dist, arrival = self:standOffTo(wide)
    want = PlanFlight.standOff(FAR_BODY_RADIUS, nil, WIDE_RADII)
    self:check(math.abs(dist - want) <= arrival * ARRIVED_SLACK, name .. "an inline spec stops at its stand-off",
        string.format("%.1f of %.1f", dist, want))
    self:check(dist > defaultStop, name .. "wider radii stop the ship farther out",
        string.format("%.1f against %.1f", dist, defaultStop))
end

function OrdersTest:caseApproachPoint()
    local name = "a point approach: "
    local ship = self:ship(Position(0, 0, 0))
    local at = Position(LEG, 0, 0)
    local station = FlightScene.spawnBody(self.world, at, STATION_RADIUS)
    station.entity:add(ConstructComponents.SpaceStation())
    self.world:update(STEP)
    local spec = ApproachSpecs.For(station.entity, nil)
    self:check(spec:isPoint(), name .. "a station has a point approach by default")
    local point = spec:pointOf(at, STATION_RADIUS, station.rb:getRot())

    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = station.entity })
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:check(within(ship, point, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the point",
        string.format("%.2f from it", ship.rb:getPos():distance(point)))
    self:finish({ order }, { ship, station })
end

function OrdersTest:caseApproachRefused()
    local name = "a refused approach: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG, 0, 0), BODY_RADIUS)
    local StandOff, Point = Enums.ApproachKind.StandOff, Enums.ApproachKind.Point
    local refused = {
        ["an approach to a point"] = { target = Position(LEG, 0, 0), approach = Enums.ApproachSpec.PlanetStandOff },
        ["radii and a gap together"] = { target = body.entity, approach = { kind = StandOff, radii = 2, gapM = 10 } },
        ["a stand-off with neither"] = { target = body.entity, approach = { kind = StandOff } },
        ["radii of zero"] = { target = body.entity, approach = { kind = StandOff, radii = 0 } },
        ["a point with no offset"] = { target = body.entity, approach = { kind = Point } },
        ["an unknown kind"] = { target = body.entity, approach = { kind = 99, radii = 2 } },
        ["an unknown id"] = { target = body.entity, approach = 9999 },
        ["a facing with no target"] = { target = body.entity, facing = {} },
    }
    for what, spec in pairs(refused) do
        self:check(not pcall(Nav.order, ship.entity, Nav.Orders.MoveTo(spec)), name .. what .. " is refused")
    end
    self:check(ship.entity:get(Components.OrderQueue) == nil or ship.entity:get(Components.OrderQueue):count() == 0,
        name .. "no order is left queued")
    self:finish({}, { ship, body })
end

---The angle between a ship's nose and the direction from the ship to a point, in radians.
---@param ship { rb: RigidBody }
---@param point Position
---@return number
local function noseOff(ship, point)
    local nose = ship.rb:getRot():getForward()
    local from = ship.rb:getPos()
    local dx, dy, dz = point.x - from.x, point.y - from.y, point.z - from.z
    local length = math.sqrt(dx * dx + dy * dy + dz * dz)
    local cosine = (nose.x * dx + nose.y * dy + nose.z * dz) / length
    return math.acos(math.max(-1, math.min(1, cosine)))
end

function OrdersTest:caseFacingHold()
    local name = "a hold with a facing: "
    local ship = self:ship(Position(0, 0, 0))
    -- To the side of the ship's initial nose, so the ship has to turn.
    local look = Position(LEG, 0, 0)
    local hold = Nav.order(ship.entity, Nav.Orders.Hold { facing = { target = look } })
    self:step()
    local before = noseOff(ship, look)
    self:check(before > FACING_TOLERANCE, name .. "the nose starts away from the point", string.format("%.2f rad", before))
    local turned = self:flyUntil(function() return noseOff(ship, look) <= FACING_TOLERANCE end, MAX_FLIGHT_S)
    self:check(turned, name .. "the nose turns to the point", string.format("%.2f rad", noseOff(ship, look)))
    self:check(Nav.status(hold) == Status.Active and within(ship, Position(0, 0, 0), NavConfig.pointArrival()),
        name .. "the ship stays on its point")

    -- A facing entity is followed as it moves.
    local body = FlightScene.spawnBody(self.world, Position(0, LEG, 0), BODY_RADIUS)
    Nav.cancel(hold)
    local second = Nav.order(ship.entity, Nav.Orders.Hold { facing = { target = body.entity } })
    self:step()
    local followed = self:flyUntil(function() return noseOff(ship, body.rb:getPos()) <= FACING_TOLERANCE end,
        MAX_FLIGHT_S)
    self:check(followed, name .. "the nose turns to an entity", string.format("%.2f rad", noseOff(ship, body.rb:getPos())))

    FlightScene.despawn(self.world, { body })
    self:step()
    self:check(Nav.status(second) == Status.Failed and Nav.failure(second) == Failure.TargetLost,
        name .. "an entity that is destroyed fails the order, TargetLost")
    self:check(not hasAgent(ship), name .. "the agent is gone")
    self:finish({ hold, second }, { ship })
end

function OrdersTest:caseFacingMoveTo()
    local name = "a move with a facing: "
    local ship = self:ship(Position(0, 0, 0))
    local goal = Position(FACING_FLIGHT, 0, 0)
    local look = Position(0, 0, -FACING_FLIGHT)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal, facing = { target = look } })

    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    local driveSeen, noseAtEnd = false, math.huge
    local reached = self:flyUntil(function()
        if drive:getState() ~= "idle" then driveSeen = true end
        noseAtEnd = noseOff(ship, look)
        return Nav.status(order) == Status.Done
    end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:check(not driveSeen, name .. "the travel drive never runs")
    self:check(within(ship, goal, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the point")
    self:check(noseAtEnd <= FACING_TOLERANCE, name .. "the nose is toward the facing point at the end",
        string.format("%.2f rad", noseAtEnd))
    self:finish({ order }, { ship })
end

---A flight long enough for a drive leg, past a planet: with a facing the plan has none, and the drive stays idle;
---without one the plan has a drive leg (the control that makes the first check mean something).
function OrdersTest:caseFacingNoDrive()
    local name = "a long flight with a facing: "
    local course = Position(PLANET_COURSE.x, PLANET_COURSE.y, PLANET_COURSE.z)
    local planet = FlightScene.buildWell(self.world,
        Position(course.x * PLANET_ALONG, course.y * PLANET_ALONG, course.z * PLANET_ALONG),
        PLANET_RADIUS, PLANET_ORBIT_SPEEDUP)
    for _, facing in ipairs({ false, true }) do
        local ship = self:ship(Position(0, 0, 0))
        local look = Position(0, 0, -FACING_FLIGHT)
        local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = course,
            facing = facing and { target = look } or nil })
        Nav.update(0)
        self:awaitPlan(order)
        local plan = planOf(order)
        self:check(plan and plan.phase == Phase.Plan, name .. "the plan has corners")
        local drives = plan and plan.follower and plan.follower:drives()
        local drive = ship.entity:get(ConstructComponents.TravelDrive)
        local driveSeen = false
        self:flyUntil(function()
            if drive:getState() ~= "idle" then driveSeen = true end
            return driveSeen
        end, SECONDS_BEFORE_CHANGE, planet)
        if facing then
            self:check(not drives and not driveSeen, name .. "with a facing, no drive leg and the drive stays idle")
        else
            self:check(drives, name .. "without a facing the plan has a drive leg")
        end
        Nav.cancel(order)
        self:step(planet)
        self:finish({ order }, { ship })
    end
    FlightScene.despawnWell(self.world, planet)
    FlightScene.despawn(self.world, { planet.planetBody })
end

function OrdersTest:caseQueue()
    local name = "a queue: "
    local ship = self:ship(Position(0, 0, 0))
    local first, second = Position(LEG / 2, 0, 0), Position(LEG / 2, LEG / 2, 0)
    local a = Nav.order(ship.entity, Nav.Orders.MoveTo { target = first })
    local b = Nav.order(ship.entity, Nav.Orders.MoveTo { target = second })
    local queue = ship.entity:get(Components.OrderQueue)
    self:check(queue:count() == 2, name .. "both orders are queued")

    local early = false
    local reached = self:flyUntil(function()
        local status = Nav.status(a)
        if status ~= Status.Done and Nav.status(b) ~= Status.Queued then early = true end
        return status == Status.Done
    end, MAX_FLIGHT_S)
    self:check(reached, name .. "the first order is done")
    self:check(not early, name .. "the second order waits for the first")
    self:check(hasAgent(ship), name .. "the ship keeps its agent between the orders")
    self:step()
    self:check(Nav.status(b) ~= Status.Queued, name .. "the second order starts when the first has ended")
    reached = self:flyUntil(function() return Nav.status(b) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the second order is done")
    self:check(within(ship, second, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the second point")
    self:check(queue:count() == 0 and not hasAgent(ship), name .. "the queue is empty and the agent gone")
    self:finish({ a, b }, { ship })
end

---Remembers a suspend or a resume that an order is to be announced with, once.
---@param order Entity
---@param kind "suspended"|"resumed"
function OrdersTest:expectTransition(order, kind)
    self.expectedTransitions[kind][order.id] = true
end

---Gives a ship a flight that has begun and is under way, to be interrupted.
---@param ship { entity: Entity, rb: RigidBody }
---@param order Entity
---@param name string
function OrdersTest:flyUntilMoving(ship, order, name)
    self:flyUntil(function() return ship.rb:getSpeed() > MOVING end, MAX_FLIGHT_S)
    self:check(ship.rb:getSpeed() > MOVING and order:get(Components.OrderFlight).begun, name .. "the order is under way")
end

function OrdersTest:casePriorityInterrupt()
    local name = "an order of higher priority: "
    local ship = self:ship(Position(0, 0, 0))
    local goal, detour = Position(LEG, 0, 0), Position(0, LEG / 2, 0)
    local routine = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal })
    self:flyUntilMoving(ship, routine, name)
    local urgent = Nav.order(ship.entity, Nav.Orders.MoveTo { target = detour, priority = Nav.Priority.Urgent })
    self:check(ship.entity:get(Components.OrderQueue):head() == urgent.id, name .. "it goes to the head of the queue")
    self:step()
    self:check(Nav.status(routine) == Status.Suspended, name .. "the order that was flying is suspended")
    self:check(planOf(routine) == nil and not routine:get(Components.OrderFlight).begun, name .. "its plan is dropped")
    self:check(Nav.status(urgent) ~= Status.Suspended and Nav.status(urgent) ~= Status.Queued, name .. "the new order flies")
    self:expectTransition(routine, "suspended")

    local early = false
    local reached = self:flyUntil(function()
        if Nav.status(urgent) ~= Status.Done and Nav.status(routine) ~= Status.Suspended then early = true end
        return Nav.status(urgent) == Status.Done
    end, MAX_FLIGHT_S)
    self:check(reached and within(ship, detour, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it is done at its point")
    self:check(not early, name .. "the first order stays suspended while it flies")
    self:step()
    self:check(Nav.status(routine) ~= Status.Suspended and Nav.status(routine) ~= Status.Queued,
        name .. "the first order resumes when it has ended")
    self:expectTransition(routine, "resumed")
    reached = self:flyUntil(function() return Nav.status(routine) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached and within(ship, goal, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it flies on from where the ship is, and is done")
    self:finish({ routine, urgent }, { ship })
end

function OrdersTest:casePriorityHold()
    local name = "a hold that is displaced: "
    local ship = self:ship(Position(0, 0, 0))
    local hold = Nav.order(ship.entity, Nav.Orders.Hold())
    self:step()
    self:check(Nav.status(hold) == Status.Active, name .. "the hold is active")
    local point = Position(LEG / 2, 0, 0)
    local urgent = Nav.order(ship.entity, Nav.Orders.MoveTo { target = point, priority = Nav.Priority.Urgent })
    self:step()
    self:check(Nav.status(hold) == Status.Suspended, name .. "an order of higher priority suspends it")
    self:expectTransition(hold, "suspended")
    local reached = self:flyUntil(function() return Nav.status(urgent) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order that displaced it is done")
    self:step()
    self:check(Nav.status(hold) == Status.Active, name .. "the hold resumes")
    self:expectTransition(hold, "resumed")
    local at = ship.rb:getPos()
    local stopped = self:flyUntil(function() return ship.rb:getSpeed() < STOPPED
        and within(ship, at, NavConfig.pointArrival() * ARRIVED_SLACK) end, MAX_FLIGHT_S)
    self:check(stopped, name .. "the ship holds where it is, not where the hold began")
    self:check(not within(ship, Position(0, 0, 0), NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it does not go back")
    Nav.cancel(hold)
    self:step()
    self:finish({ hold, urgent }, { ship })
end

function OrdersTest:casePriorityPatrol()
    local name = "a patrol that is displaced: "
    local ship = self:ship(Position(0, 0, 0))
    local half = LEG / 2
    local points = { Position(half, 0, 0), Position(half, half, 0), Position(0, half, 0) }
    local patrol = Nav.order(ship.entity, Nav.Orders.Patrol { points = points })
    local spec = patrol:get(Components.PatrolOrder)
    self:flyUntil(function() return spec:getIndex() == 2 end, MAX_FLIGHT_S)
    self:check(spec:getIndex() == 2, name .. "the patrol is on its second point")
    local urgent = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(half, -half, 0), priority = Nav.Priority.Combat })
    self:step()
    self:check(Nav.status(patrol) == Status.Suspended and spec:getIndex() == 2, name .. "it is suspended and keeps its point")
    self:expectTransition(patrol, "suspended")
    self:flyUntil(function() return Nav.status(urgent) == Status.Done end, MAX_FLIGHT_S)
    self:step()
    self:expectTransition(patrol, "resumed")
    self:check(Nav.status(patrol) ~= Status.Suspended and spec:getIndex() == 2, name .. "it resumes at the same point")
    local reached = self:flyUntil(function() return Nav.status(patrol) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached and within(ship, points[3], NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it goes on to the last point")
    self:finish({ patrol, urgent }, { ship })
end

function OrdersTest:casePriorityQueueOrder()
    local name = "the order of a queue by priority: "
    local ship = self:ship(Position(0, 0, 0))
    local function moveTo(y, priority)
        return Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, y, 0), priority = priority })
    end
    local a = moveTo(0, Nav.Priority.Routine)
    local queue = ship.entity:get(Components.OrderQueue)
    self:flyUntilMoving(ship, a, name)
    local c = moveTo(1, Nav.Priority.Routine)
    local b = moveTo(2, Nav.Priority.Combat)
    local d = moveTo(3, Nav.Priority.Combat)
    local e = moveTo(4, Nav.Priority.Urgent)
    local ids = queue:ids()
    local wanted = { b.id, d.id, e.id, a.id, c.id }
    local ok = #ids == #wanted
    for i, id in ipairs(wanted) do ok = ok and ids[i] == id end
    self:check(ok, name .. "higher first, and in the order they were given within a level", table.concat(ids, ","))
    self:step()
    self:expectTransition(a, "suspended")
    self:check(Nav.status(a) == Status.Suspended, name .. "the running order is suspended")
    self:check(Nav.status(d) == Status.Queued and Nav.status(c) == Status.Queued, name .. "an order of the same priority waits behind the head")

    -- A suspended order that is cancelled ends, and doesn't resume.
    Nav.cancel(a)
    self:step()
    self:check(Nav.status(a) == Status.Cancelled, name .. "a suspended order can be cancelled")
    self:check(queue:count() == 4, name .. "it has left the queue")
    for _, order in ipairs({ b, d, e, c }) do Nav.cancel(order) end
    self:step()
    self:check(queue:count() == 0 and not hasAgent(ship), name .. "the rest are cancelled")
    self:finish({ a, b, c, d, e }, { ship })
end

function OrdersTest:casePriorityRefused()
    local name = "a priority that is not a level: "
    local ship = self:ship(Position(0, 0, 0))
    for what, priority in pairs({ zero = 0, high = 99, text = "Urgent" }) do
        self:check(not pcall(Nav.order, ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0), priority = priority }),
            name .. what .. " is refused")
    end
    local queue = ship.entity:get(Components.OrderQueue)
    self:check(queue == nil or queue:count() == 0, name .. "no order is left behind")
    self:finish({}, { ship })
end

function OrdersTest:caseClaim()
    local name = "a claimed ship: "
    local ship = self:ship(Position(0, 0, 0))
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local goal = Position(LEG, 0, 0)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal })
    self:check(Nav.controlOwner(ship.entity) == Owner.Orders, name .. "orders own a ship that has an order")
    self:flyUntilMoving(ship, order, name)

    Nav.claim(ship.entity)
    self:check(Nav.controlOwner(ship.entity) == Owner.Claim, name .. "the claimer owns it")
    self:check(axesAreZero(input) and input:getDriveSpeedCap() == nil
        and input:getDriveRequest() == Enums.TravelDriveRequest.Disengage, name .. "the input is reset at the handover")
    input:setThrustForward(1)
    self:step()
    self:check(Nav.status(order) == Status.Suspended, name .. "the running order is suspended")
    self:expectTransition(order, "suspended")
    self:check(planOf(order) == nil and not hasAgent(ship), name .. "its plan is dropped and the ship has left the world")
    self:flyUntil(function() return false end, 1)
    self:check(input:getThrustForward() == 1 and input:getYaw() == 0, name .. "the orders leave the input to the claimer")
    self:check(Nav.status(order) == Status.Suspended and not hasAgent(ship), name .. "the order waits")

    input:clearAxes()
    Nav.unclaim(ship.entity)
    self:check(Nav.controlOwner(ship.entity) == Owner.Orders, name .. "orders own the ship again")
    self:step()
    self:check(Nav.status(order) ~= Status.Suspended and hasAgent(ship), name .. "the order resumes")
    self:expectTransition(order, "resumed")
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached and within(ship, goal, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it flies on from where the ship is, and is done")
    self:finish({ order }, { ship })
end

function OrdersTest:caseManualShip()
    local name = "a ship the player pilots: "
    local ship = self:ship(Position(0, 0, 0))
    ControlOwnerSystem:setManual(ship.entity, true)
    local goal = Position(LEG, 0, 0)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal })
    self:check(Nav.controlOwner(ship.entity) == Owner.Manual, name .. "manual input owns it")
    self:flyUntil(function() return false end, 1)
    self:check(Nav.status(order) == Status.Queued and not hasAgent(ship), name .. "its order waits, with no agent")
    self:check(ship.rb:getSpeed() < STOPPED, name .. "the ship is not flown")

    ControlOwnerSystem:setManual(ship.entity, false)
    self:check(Nav.controlOwner(ship.entity) == Owner.Orders, name .. "orders own it when the player leaves it")
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached and within(ship, goal, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the order is flown then")
    self:finish({ order }, { ship })
end

function OrdersTest:caseOwnerPrecedence()
    local name = "who owns the controls: "
    local ship = self:ship(Position(0, 0, 0))
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local function owner() return Nav.controlOwner(ship.entity) end
    self:check(owner() == Owner.None, name .. "nobody, for a ship nothing asks for")
    Nav.claim(ship.entity)
    input:setYaw(0.5)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    self:check(owner() == Owner.Claim and input:getYaw() == 0.5, name .. "a claim holds against an order given later")
    ControlOwnerSystem:setManual(ship.entity, true)
    self:check(owner() == Owner.Manual, name .. "manual input over a claim")
    ControlOwnerSystem:setManual(ship.entity, false)
    self:check(owner() == Owner.Claim, name .. "the claim again when the player leaves")
    self:check(not pcall(Nav.claim, ship.entity), name .. "a ship claimed twice is refused")
    Nav.unclaim(ship.entity)
    self:check(owner() == Owner.Orders, name .. "orders when nothing stronger is left")
    self:check(not pcall(Nav.unclaim, ship.entity), name .. "an unclaim of a ship that is not claimed is refused")

    self:step()
    input:setYaw(0.5)
    Nav.cancel(order)
    self:step()
    self:check(Nav.status(order) == Status.Cancelled and owner() == Owner.None, name .. "nobody when the last order has ended")
    self:check(axesAreZero(input), name .. "the input is reset then")
    Nav.claim(ship.entity)
    input:setYaw(0.5)
    Nav.unclaim(ship.entity)
    self:check(owner() == Owner.None and axesAreZero(input), name .. "a claim on a ship with no orders gives it back reset")
    self:finish({ order }, { ship })
end

function OrdersTest:caseClaimedPriority()
    local name = "orders given to a claimed ship: "
    local ship = self:ship(Position(0, 0, 0))
    Nav.claim(ship.entity)
    local routine = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    local detour = Position(0, LEG / 2, 0)
    local combat = Nav.order(ship.entity, Nav.Orders.MoveTo { target = detour, priority = Nav.Priority.Combat })
    self:flyUntil(function() return false end, 1)
    self:check(Nav.status(routine) == Status.Queued and Nav.status(combat) == Status.Queued and not hasAgent(ship),
        name .. "they wait, both queued")
    Nav.unclaim(ship.entity)
    self:step()
    self:check(Nav.status(combat) ~= Status.Queued and Nav.status(routine) == Status.Queued, name .. "the one of higher priority flies first")
    local reached = self:flyUntil(function() return Nav.status(combat) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached and within(ship, detour, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "it is done at its point")
    reached = self:flyUntil(function() return Nav.status(routine) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the other follows")
    self:finish({ routine, combat }, { ship })
end

function OrdersTest:casePatrol(loop)
    local name = loop and "a patrol with a loop: " or "a patrol without a loop: "
    local ship = self:ship(Position(0, 0, 0))
    local half = LEG / 2
    local points = { Position(half, 0, 0), Position(half, half, 0), Position(0, half, 0) }
    local order = Nav.order(ship.entity, Nav.Orders.Patrol { points = points, loop = loop })

    local seen, wrapped = {}, false
    local function observe()
        local index = Nav.progress(order).waypointIndex
        if index then
            if seen[#points] and index == 1 then wrapped = true end
            seen[index] = true
        end
        if loop then return wrapped end
        return Nav.status(order) == Status.Done
    end
    local reached = self:flyUntil(observe, MAX_FLIGHT_S)
    self:check(reached, name .. (loop and "the ship completes a lap" or "the order is done"))
    self:check(seen[1] and seen[2] and seen[3], name .. "every point is flown to in turn")
    if loop then
        -- The next leg is planned first, so the order may be planning again.
        self:check(Nav.status(order) == Status.Active or Nav.status(order) == Status.Planning,
            name .. "the order goes on after a lap")
        Nav.cancel(order)
        self:step()
        self:check(Nav.status(order) == Status.Cancelled, name .. "cancelled")
        self:check(not hasAgent(ship), name .. "the agent is gone")
        self:check(axesAreZero(ship.entity:get(ConstructComponents.ControlInput)), name .. "no axes are left")
    else
        self:check(within(ship, points[#points], NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the last point")
    end
    self:finish({ order }, { ship })
end

function OrdersTest:caseHold()
    local name = "a hold: "
    local ship = self:ship(Position(0, 0, 0))
    local run = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    self:flyUntil(function() return ship.rb:getSpeed() > MOVING end, MAX_FLIGHT_S)
    self:check(ship.rb:getSpeed() > MOVING, name .. "the ship is moving")
    Nav.cancel(run)
    local hold = Nav.order(ship.entity, Nav.Orders.Hold())
    self:step()
    self:check(Nav.status(run) == Status.Cancelled, name .. "the order before it is cancelled")
    self:check(Nav.status(hold) == Status.Active, name .. "the hold is active")
    local at = ship.rb:getPos()

    local farthest = 0
    self:flyUntil(function()
        farthest = math.max(farthest, ship.rb:getPos():distance(at))
        return false
    end, SECONDS_BEFORE_CHANGE)
    self:check(farthest > ship.rb:getBoundingRadius(), name .. "the ship carries on a little before it turns",
        string.format("farthest %.4f, speed was above %.2f", farthest, MOVING))
    local stopped = self:flyUntil(function() return ship.rb:getSpeed() < STOPPED
        and within(ship, at, NavConfig.pointArrival() * ARRIVED_SLACK) end, MAX_FLIGHT_S)
    self:check(stopped, name .. "the ship comes back to its point and stops")
    self:check(Nav.status(hold) == Status.Active, name .. "a hold doesn't end by itself")
    Nav.cancel(hold)
    self:step()
    self:check(Nav.status(hold) == Status.Cancelled and not hasAgent(ship), name .. "cancelled, the agent is gone")
    self:finish({ run, hold }, { ship })
end

function OrdersTest:caseCancelQueued()
    local name = "a cancelled queued order: "
    local ship = self:ship(Position(0, 0, 0))
    local a = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    local b = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(0, LEG, 0) })
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE / 2)
    Nav.cancel(b)
    self:step()
    self:check(Nav.status(b) == Status.Cancelled, name .. "cancelled")
    self:check(not b:get(Components.OrderFlight).begun, name .. "it never began")
    self:check(Nav.status(a) ~= Status.Cancelled and hasAgent(ship), name .. "the order before it goes on")
    self:check(ship.entity:get(Components.OrderQueue):count() == 1, name .. "it has left the queue")
    Nav.cancel(a)
    self:step()
    self:check(Nav.status(a) == Status.Cancelled and not hasAgent(ship), name .. "the running one is cancelled too")
    self:finish({ a, b }, { ship })
end

function OrdersTest:caseRetarget()
    local name = "a change of target: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE)
    local before = planOf(order)
    self:check(before ~= nil, name .. "the order has a plan")

    local newGoal = Position(0, 0, -LEG)
    Nav.retarget(order, newGoal)
    self:step()
    self:check(planOf(order) ~= before, name .. "the path is planned again")
    self:awaitPlan(order)
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:check(within(ship, newGoal, NavConfig.pointArrival() * ARRIVED_SLACK), name .. "the ship is at the new target")
    self:finish({ order }, { ship })
end

function OrdersTest:caseCostProfileChange()
    local name = "a change of cost profile: "
    local ship = self:ship(Position(0, 0, 0))
    local profile = ship.entity:add(Components.CostProfile(CostProfileDefs:get(Enums.CostProfile.Standard)))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE)
    local before = planOf(order)

    self:step()
    self:check(planOf(order) == before, name .. "the plan stands while nothing changes")
    profile:setProfile(CostProfileDefs:get(Enums.CostProfile.Cautious))
    self:step()
    local flight = order:get(Components.OrderFlight)
    self:check(planOf(order) == before and flight.pending ~= nil, name .. "the ship flies its plan while the new one is asked for")
    -- The planner answers in its own time: wait for the work, for at most the plan wait.
    local start = InstantTime.Now()
    while planOf(order) == before and InstantTime.Now():durationSince(start) < PLAN_WAIT_S * MS_PER_S do
        Nav.update(0)
    end
    self:check(planOf(order) ~= before, name .. "the path is planned again")
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:finish({ order }, { ship })
end

function OrdersTest:caseTargetDestroyed()
    local name = "a destroyed target: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG, 0, LEG / 2), BODY_RADIUS)
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity,
        approach = { kind = Enums.ApproachKind.StandOff, gapM = GAP_M } })
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE / 2)
    self:check(Nav.status(order) == Status.Planning or Nav.status(order) == Status.Active, name .. "the order runs")

    FlightScene.despawn(self.world, { body })
    self:step()
    self:check(Nav.status(order) == Status.Failed, name .. "the order fails")
    self:check(Nav.failure(order) == Failure.TargetLost, name .. "because the target is lost")
    self:check(not hasAgent(ship), name .. "the agent is gone")
    self:finish({ order }, { ship })
end

function OrdersTest:caseShipDestroyed()
    local name = "a destroyed ship: "
    local ship = self:ship(Position(0, 0, 0))
    local other = self:ship(Position(0, LEG, 0))
    local lost = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    local kept = Nav.order(other.entity, Nav.Orders.MoveTo { target = Position(LEG, LEG, 0) })
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE / 2)

    FlightScene.despawn(self.world, { ship })
    self:step()
    self:check(Nav.status(lost) == Status.Failed and Nav.failure(lost) == Failure.ShipLost, name .. "the order fails, ShipLost")
    self:check(Nav.status(kept) ~= Status.Failed and hasAgent(other), name .. "another ship's order goes on")
    local agents = 0
    for _ in worldComponent():slots() do agents = agents + 1 end
    self:check(agents == 1, name .. "the lost ship's agent is gone", tostring(agents))
    Nav.cancel(kept)
    self:step()
    self:finish({ lost, kept }, { other })
end

function OrdersTest:caseRelease()
    local name = "release: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    self:step()
    self:check(not pcall(Nav.release, order), name .. "an order that has not ended is not released")
    self:check(Registry:getEntityById(order.id) ~= nil, name .. "it is still there")
    self:check(not pcall(Nav.attach, self.world), name .. "a second navigation world is refused")
    Nav.cancel(order)
    self:step()
    self:finish({ order }, { ship })
end

---The last case: the world's detach cancels the orders that run.
function OrdersTest:caseDetach()
    local name = "detach: "
    local ship = self:ship(Position(0, 0, 0))
    local run = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    local hold = Nav.order(ship.entity, Nav.Orders.Hold())
    self:flyUntil(function() return false end, 1)
    Nav.detach()
    self:check(Nav.status(run) == Status.Cancelled and Nav.status(hold) == Status.Cancelled,
        name .. "the orders are cancelled")
    self:check(not hasAgent(ship), name .. "the agent is gone")
    self:finish({ run, hold }, { ship })
end

---The ship's place about a centre, in the plane of a circle: along `first`, and along the way round.
---@param ship { rb: RigidBody }
---@param centre Position
---@param normal { x: number, y: number, z: number }
---@param first { x: number, y: number, z: number }|nil The direction the angle is counted from; the ship's own when nil
---@return number angle Radians, anticlockwise about the normal
---@return number radius Distance within the plane
---@return number height Distance from the plane
---@return { x: number, y: number, z: number } first
local function circlePlace(ship, centre, normal, first)
    local at = ship.rb:getPos()
    local rel = { x = at.x - centre.x, y = at.y - centre.y, z = at.z - centre.z }
    local height = rel.x * normal.x + rel.y * normal.y + rel.z * normal.z
    local flat = { x = rel.x - normal.x * height, y = rel.y - normal.y * height, z = rel.z - normal.z * height }
    local flatLength = math.sqrt(flat.x ^ 2 + flat.y ^ 2 + flat.z ^ 2)
    first = first or { x = flat.x / flatLength, y = flat.y / flatLength, z = flat.z / flatLength }
    local second = {
        x = normal.y * first.z - normal.z * first.y,
        y = normal.z * first.x - normal.x * first.z,
        z = normal.x * first.y - normal.y * first.x,
    }
    local angle = math.atan2(flat.x * second.x + flat.y * second.y + flat.z * second.z,
        flat.x * first.x + flat.y * first.y + flat.z * first.z)
    return angle, flatLength, math.abs(height), first
end

-- An orbit: the altitude asked for, the share of the radius the ship may stray from the circle and from its plane
-- once on it, and the turn it is to make (a circle takes minutes: its speed is set by the thrust).
local ORBIT_ALTITUDE   = 2
local ORBIT_STRAY      = 0.25
local ORBIT_TURN       = math.pi / 2
local ORBIT_TURN_S     = 600
local ORBIT_BODY_LEGS  = 3.5
local WELL_PLANET_RADIUS = 100
local WELL_SPEEDUP     = 0.1
local WELL_START       = 300
-- The planet's share of the orbit radius it has moved once the ship has been round a quarter turn.
local WELL_MOVED       = 0.1
-- The leader of a follower (the size of a ship) and where the slot is in its axes, in metres.
local LEADER_RADIUS = 0.005
local LEADER_SPEED_SHARE = 0.3
local SLOT_M = { x = 2000, y = 0, z = 3000 }
-- A leader moved this share of the join range stays near the slot the plan was made for; the far one doesn't.
local SLOT_NEAR_SHARE = 0.2
local SLOT_FAR_SHARE  = 1.2
local LEADER_BEYOND   = 1.2
local LEADER_FAR      = 2.4
local HOLD_S          = 5
local SETTLE_S        = 5
local SPEED_MATCH     = 0.25

---Flies an Orbit order to its circle and round it for a turn, and checks the ship holds the circle and
---its plane, anticlockwise about the normal.
---@param name string
---@param order Entity
---@param ship { entity: Entity, rb: RigidBody }
---@param centreOf fun(): Position Where the centre is now
---@param radius gameunit
---@param move fun()|nil Called before each step, to move the world
---@param well table|nil
function OrdersTest:flyOrbit(name, order, ship, centreOf, radius, move, well)
    local flight = order:get(Components.OrderFlight)
    local planSeen = false
    Nav.update(0)
    self:awaitPlan(order)
    local reached = self:flyUntil(function()
        if move then move() end
        planSeen = planSeen or planOf(order) ~= nil
        return flight.tracking
    end, MAX_FLIGHT_S, well)
    self:check(reached, name .. "the ship reaches the circle and the order tracks", string.format("status %s, plan %s, %.1f from the centre, speed %.2f",
        tostring(Nav.status(order)), tostring(planOf(order) and planOf(order).phase), ship.rb:getPos():distance(centreOf()), ship.rb:getSpeed()))
    self:check(planSeen, name .. "the ship flew a plan to the circle")
    self:check(planOf(order) == nil, name .. "the plan is dropped for the orbit")
    self:check(Nav.status(order) == Status.Active, name .. "the order is active")
    local normal = order:get(Components.OrbitOrder):getChosenNormal()
    self:check(normal ~= nil, name .. "the order has its plane")
    if not normal then return end

    -- The ship comes onto the circle with the velocity of its approach, and settles into its plane.
    local _, _, _, first = circlePlace(ship, centreOf(), normal, nil)
    local function straying()
        local _, flat, height = circlePlace(ship, centreOf(), normal, first)
        return math.max(math.abs(flat - radius), height)
    end
    local settled = self:flyUntil(function()
        if move then move() end
        return straying() <= ORBIT_STRAY * radius
    end, ORBIT_TURN_S, well)
    self:check(settled, name .. "the ship settles on the circle and its plane", string.format("off by %.1f of %.1f", straying(), radius))

    local previous, swept, worst = nil, 0, 0
    local turned = self:flyUntil(function()
        if move then move() end
        local angle = circlePlace(ship, centreOf(), normal, first)
        if previous then
            local delta = angle - previous
            if delta > math.pi then delta = delta - 2 * math.pi elseif delta < -math.pi then delta = delta + 2 * math.pi end
            swept = swept + delta
        end
        previous = angle
        worst = math.max(worst, straying())
        return swept >= ORBIT_TURN
    end, ORBIT_TURN_S, well)
    self:check(turned, name .. "the ship goes round anticlockwise about the normal", string.format("swept %.2f", swept))
    self:check(worst <= ORBIT_STRAY * radius, name .. "the ship holds the circle and its plane",
        string.format("off by %.1f of %.1f", worst, radius))
    local progress = Nav.progress(order)
    self:check(progress.eta == 0 and progress.distanceLeft ~= nil, name .. "the progress is the distance from the circle")
end

---Cancels a tracking order and checks that the ship is let go.
---@param name string
---@param order Entity
---@param ship { entity: Entity }
function OrdersTest:cancelTracking(name, order, ship)
    Nav.cancel(order)
    self:step()
    self:check(Nav.status(order) == Status.Cancelled, name .. "cancelled")
    self:check(not hasAgent(ship), name .. "the agent is gone")
    self:check(axesAreZero(ship.entity:get(ConstructComponents.ControlInput)), name .. "no axes are left")
end

function OrdersTest:caseOrbit(named)
    local name = named and "an orbit about a named normal: " or "an orbit in the ship's plane: "
    local ship = self:ship(Position(0, 0, 0))
    local centre = Position(LEG * ORBIT_BODY_LEGS, LEG / 4, 0)
    local body = FlightScene.spawnBody(self.world, centre, FAR_BODY_RADIUS)
    self.world:update(STEP)
    local normal = Vec3d(0.3, 0.2, 0.9)
    local order = Nav.order(ship.entity, Nav.Orders.Orbit {
        target = body.entity, altitude = ORBIT_ALTITUDE, normal = named and normal or nil })
    local radius = PlanFlight.standOff(FAR_BODY_RADIUS, nil, ORBIT_ALTITUDE)
    self:flyOrbit(name, order, ship, function() return centre end, radius)
    if named then
        local chosen = order:get(Components.OrbitOrder):getChosenNormal()
        local length = math.sqrt(normal.x ^ 2 + normal.y ^ 2 + normal.z ^ 2)
        self:check(chosen and math.abs(chosen.x - normal.x / length) < 1e-9 and math.abs(chosen.z - normal.z / length) < 1e-9,
            name .. "the plane is the named one")
    end
    self:cancelTracking(name, order, ship)
    self:finish({ order }, { ship, body })
end

---An orbit about a planet that moves in its gravity well: the ship rides the well, and the circle follows the planet.
function OrdersTest:caseOrbitInWell()
    local name = "an orbit about a planet in a well: "
    local planetPos = Position(LEG * ORBIT_BODY_LEGS, 0, 0)
    local planet = FlightScene.buildWell(self.world, planetPos, WELL_PLANET_RADIUS, WELL_SPEEDUP)
    local ship = FlightScene.spawnShip(self.world, Position(planetPos.x + WELL_START, planetPos.y, planetPos.z),
        { inWell = true })
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.Orbit {
        target = planet.planetBody.entity, normal = Vec3d(0, 1, 0) })
    local radius = PlanFlight.standOff(WELL_PLANET_RADIUS, nil, NavConfig.standOffRadii)
    local start = planet.planetBody.rb:getPos()
    self:flyOrbit(name, order, ship, function() return planet.planetBody.rb:getPos() end, radius, nil, planet)
    self:check(planet.planetBody.rb:getPos():distance(start) > WELL_MOVED * radius, name .. "the planet has moved",
        string.format("%.1f", planet.planetBody.rb:getPos():distance(start)))
    self:cancelTracking(name, order, ship)
    FlightScene.despawnWell(self.world, planet)
    FlightScene.despawn(self.world, { planet.planetBody })
    self:finish({ order }, { ship })
end

---A kinematic leader, moved by hand: `to` sets its place for the next step.
---@param leader { rb: RigidBody }
---@param position Position
local function placeLeader(leader, position)
    leader.rb:setKinematicTarget(position, leader.rb:getRot())
end

---The slot in the leader's axes, in game units.
---@param leader { rb: RigidBody }
---@return Position
local function slotOf(leader)
    local rotation = leader.rb:getRot()
    local right, up, forward = rotation:getRight(), rotation:getUp(), rotation:getForward()
    local x, y, z = ScaleConfig:toGameUnits(SLOT_M.x, "starSystem"), ScaleConfig:toGameUnits(SLOT_M.y, "starSystem"),
        ScaleConfig:toGameUnits(SLOT_M.z, "starSystem")
    local at = leader.rb:getPos()
    return Position(at.x + right.x * x + up.x * y + forward.x * z, at.y + right.y * x + up.y * y + forward.y * z,
        at.z + right.z * x + up.z * y + forward.z * z)
end

function OrdersTest:caseFollowInRange()
    local name = "a follower in range: "
    local top = FlightProfileRegistry:get(Enums.FlightProfile.Solo).maxSpeed
    local ship = self:ship(Position(0, 0, 0))
    local leader = FlightScene.spawnBody(self.world, Position(LEG, 0, 0), LEADER_RADIUS)
    self.world:update(STEP)
    local velocity = { x = 0, y = 0, z = -LEADER_SPEED_SHARE * top }
    local function moveLeader()
        local at = leader.rb:getPos()
        placeLeader(leader, Position(at.x + velocity.x * STEP, at.y + velocity.y * STEP, at.z + velocity.z * STEP))
    end
    local order = Nav.order(ship.entity, Nav.Orders.Follow {
        target = leader.entity, offset = Vec3d(SLOT_M.x, SLOT_M.y, SLOT_M.z) })
    Nav.update(0)
    local flight = order:get(Components.OrderFlight)
    self:check(flight.tracking and planOf(order) == nil, name .. "within the join range there is no plan")
    self:check(Nav.status(order) == Status.Active, name .. "the order is active")

    local arrival = NavConfig.pointArrival() * ARRIVED_SLACK
    local onSlot = self:flyUntil(function()
        moveLeader()
        return within(ship, slotOf(leader), arrival)
    end, MAX_FLIGHT_S)
    self:check(onSlot, name .. "the ship reaches its slot")
    -- The ship reaches the slot with the speed of its approach, and settles onto the leader's velocity.
    local worst, speedOff, frames = 0, 0, 0
    self:flyUntil(function()
        moveLeader()
        frames = frames + 1
        worst = math.max(worst, ship.rb:getPos():distance(slotOf(leader)))
        if frames * STEP > SETTLE_S then
            speedOff = math.max(speedOff, (ship.rb:getVelocity() - leader.rb:getVelocity()):length())
        end
        return false
    end, SETTLE_S + HOLD_S)
    self:check(worst <= arrival, name .. "the ship keeps its slot as the leader moves", string.format("off by %.1f", worst))
    self:check(speedOff <= SPEED_MATCH * LEADER_SPEED_SHARE * top, name .. "the ship matches the leader's velocity",
        string.format("off by %.2f of %.2f", speedOff, LEADER_SPEED_SHARE * top))
    self:check(planOf(order) == nil and flight.tracking and Nav.status(order) == Status.Active,
        name .. "it never planned")
    self:check(Nav.progress(order).eta == 0, name .. "the progress is the distance from the slot")
    self:cancelTracking(name, order, ship)
    self:finish({ order }, { ship, leader })
end

function OrdersTest:caseFollowOutOfRange()
    local name = "a follower out of range: "
    local join = PlanFlight.driveMinLength(0)
    local ship = self:ship(Position(0, 0, 0))
    local leader = FlightScene.spawnBody(self.world, Position(join * LEADER_BEYOND, 0, 0), LEADER_RADIUS)
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.Follow {
        target = leader.entity, offset = Vec3d(SLOT_M.x, SLOT_M.y, SLOT_M.z) })
    local flight = order:get(Components.OrderFlight)
    Nav.update(0)
    self:check(not flight.tracking and planOf(order) ~= nil, name .. "beyond the join range the ship plans")

    local before = planOf(order)
    leader.rb:setPos(Position(join * (LEADER_BEYOND + SLOT_NEAR_SHARE), 0, 0))
    self:step()
    self:check(planOf(order) == before, name .. "a slot that moved a little keeps the plan")
    leader.rb:setPos(Position(join * LEADER_BEYOND, join * SLOT_FAR_SHARE, 0))
    self:step()
    self:check(planOf(order) ~= before and planOf(order) ~= nil, name .. "a slot that moved far makes a new plan")

    self:awaitPlan(order)
    local joined = self:flyUntil(function() return flight.tracking end, MAX_FLIGHT_S)
    self:check(joined, name .. "the ship joins the formation")
    self:check(planOf(order) == nil, name .. "the plan is dropped at the join")
    -- Within the join range the travel drive would not run either, so the rest is a flight at the ship's own speed.
    local gap = ship.rb:getPos():distance(slotOf(leader))
    self:check(gap <= join, name .. "the ship joins within the join range", string.format("%.1f of %.1f", gap, join))
    self:flyUntil(function() return false end, SETTLE_S)
    self:check(ship.rb:getPos():distance(slotOf(leader)) < gap, name .. "the ship closes on its slot")
    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    self:check(Nav.status(order) == Status.Active and drive:getState() == "idle", name .. "the order is active, the drive is off")

    local at = ship.rb:getPos()
    leader.rb:setPos(Position(at.x + join * LEADER_FAR, at.y, at.z))
    self:step()
    self:check(not flight.tracking and planOf(order) ~= nil, name .. "a leader that leaves too far for the join is planned to")
    self:check(Nav.status(order) == Status.Planning or Nav.status(order) == Status.Active, name .. "the order goes on")
    Nav.cancel(order)
    self:step()
    self:check(Nav.status(order) == Status.Cancelled and not hasAgent(ship), name .. "cancelled, the agent is gone")
    self:finish({ order }, { ship, leader })
end

---A lost target ends an Orbit or Follow order that is tracking, and one that has not reached its goal.
function OrdersTest:caseTrackTargetLost()
    local name = "a destroyed target of orbit and follow: "
    local orbiter = self:ship(Position(0, 0, 0))
    local follower = self:ship(Position(0, LEG, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG * ORBIT_BODY_LEGS, 0, 0), FAR_BODY_RADIUS)
    local leader = FlightScene.spawnBody(self.world, Position(LEG, LEG, 0), LEADER_RADIUS)
    self.world:update(STEP)
    local orbit = Nav.order(orbiter.entity, Nav.Orders.Orbit { target = body.entity })
    local follow = Nav.order(follower.entity, Nav.Orders.Follow { target = leader.entity, offset = Vec3d(0, 0, SLOT_M.z) })
    self:step()
    self:check(follow:get(Components.OrderFlight).tracking, name .. "the follower tracks")
    self:check(not orbit:get(Components.OrderFlight).tracking, name .. "the orbiter is still on its way")

    FlightScene.despawn(self.world, { body, leader })
    self:step()
    for label, order in pairs({ orbit = orbit, follow = follow }) do
        self:check(Nav.status(order) == Status.Failed and Nav.failure(order) == Failure.TargetLost,
            name .. label .. " fails, TargetLost")
    end
    self:check(not hasAgent(orbiter) and not hasAgent(follower), name .. "the agents are gone")
    self:finish({ orbit, follow }, { orbiter, follower })
end

---Specs that are refused leave nothing behind.
function OrdersTest:caseTrackRefused()
    local name = "refused orbit and follow specs: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG, 0, 0), FAR_BODY_RADIUS)
    local bare = Registry:createEntity()
    self.world:update(STEP)
    local refused = {
        ["an altitude of 1"] = Nav.Orders.Orbit { target = body.entity, altitude = 1 },
        ["an altitude below 1"] = Nav.Orders.Orbit { target = body.entity, altitude = 0.5 },
        ["a zero normal"] = Nav.Orders.Orbit { target = body.entity, normal = Vec3d(0, 0, 0) },
        ["an orbit of an entity with no body"] = Nav.Orders.Orbit { target = bare },
        ["an orbit of a point"] = Nav.Orders.Orbit { target = Position(0, 0, 0) },
        ["a follow with no offset"] = Nav.Orders.Follow { target = body.entity },
        ["a follow of an entity with no body"] = Nav.Orders.Follow { target = bare, offset = Vec3d(0, 0, 1) },
        ["a follow of a point"] = Nav.Orders.Follow { target = Position(0, 0, 0), offset = Vec3d(0, 0, 1) },
    }
    for what, spec in pairs(refused) do
        self:check(not pcall(Nav.order, ship.entity, spec), name .. what)
    end
    local queue = ship.entity:get(Components.OrderQueue)
    self:check(queue == nil or queue:count() == 0, name .. "no order is left behind")
    Registry:destroyEntity(bare)
    self:finish({}, { ship, body })
end

---Checks the events once they have been delivered: one per ended order, of the kind of its end.
function OrdersTest:checkEvents()
    local expectedCount, receivedCount = 0, 0
    for id, status in pairs(self.expected) do
        expectedCount = expectedCount + 1
        self:check(self.received[status][id] == true, "the event of order " .. id .. " arrives with its id")
    end
    for _, ids in pairs(self.received) do
        for _ in pairs(ids) do receivedCount = receivedCount + 1 end
    end
    self:check(receivedCount == expectedCount, "one event per ended order",
        string.format("%d of %d", receivedCount, expectedCount))
    self:check(self.duplicates == 0, "no event arrives twice", tostring(self.duplicates))
    for kind, received in pairs(self.transitions) do
        for id in pairs(self.expectedTransitions[kind]) do
            self:check(received[id] == 1, "order " .. id .. " is announced " .. kind .. " once", tostring(received[id]))
        end
        for id in pairs(received) do
            self:check(self.expectedTransitions[kind][id] == true, "order " .. id .. " is " .. kind .. " only when it should be")
        end
    end
    self:check(expectedCount > 0, "orders ended")
end

---Counts the events as they arrive, per kind of end.
function OrdersTest:listen()
    self.received = { [Status.Done] = {}, [Status.Failed] = {}, [Status.Cancelled] = {} }
    self.duplicates = 0
    self.transitions = { suspended = {}, resumed = {} }
    self.expectedTransitions = { suspended = {}, resumed = {} }
    for kind, name in pairs({ suspended = "NavOrderSuspended", resumed = "NavOrderResumed" }) do
        EventBus:subscribe(Event[name], nil, function(_, _, id)
            self.transitions[kind][id] = (self.transitions[kind][id] or 0) + 1
        end)
    end
    local names = {
        [Status.Done] = "NavOrderDone", [Status.Failed] = "NavOrderFailed", [Status.Cancelled] = "NavOrderCancelled",
    }
    for status, name in pairs(names) do
        EventBus:subscribe(Event[name], nil, function(_, _, id)
            if self.received[status][id] then self.duplicates = self.duplicates + 1 end
            self.received[status][id] = true
        end)
    end
end

function OrdersTest:onInit()
    self.passed, self.failed = 0, 0
    self.expected = {}
    self.frames = 0

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    Nav.attach(self.world)
    self:listen()

    self:caseMoveToPastPlanet()
    self:caseMoveToBody()
    self:caseApproachStandOff()
    self:caseApproachPoint()
    self:caseApproachRefused()
    self:caseFacingHold()
    self:caseFacingMoveTo()
    self:caseFacingNoDrive()
    self:caseQueue()
    self:casePriorityInterrupt()
    self:casePriorityHold()
    self:casePriorityPatrol()
    self:casePriorityQueueOrder()
    self:casePriorityRefused()
    self:caseClaim()
    self:caseManualShip()
    self:caseOwnerPrecedence()
    self:caseClaimedPriority()
    self:casePatrol(false)
    self:casePatrol(true)
    self:caseHold()
    self:caseOrbit(true)
    self:caseOrbit(false)
    self:caseOrbitInWell()
    self:caseFollowInRange()
    self:caseFollowOutOfRange()
    self:caseTrackTargetLost()
    self:caseTrackRefused()
    self:caseCancelQueued()
    self:caseRetarget()
    self:caseCostProfileChange()
    self:caseTargetDestroyed()
    self:caseShipDestroyed()
    self:caseRelease()
    self:caseDetach()

    -- The events are delivered in the frames that follow; the world is gone, so no order runs in them.
    EventBus:subscribe(Event.PostInput, self, function()
        self.frames = self.frames + 1
        if self.frames < EVENT_FRAMES and self.frames < MAX_FRAMES then return end
        self:checkEvents()
        Log.Info("NavigationOrdersTest: %d checks passed, %d failed", self.passed, self.failed)
        self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
    end)
end

return OrdersTest
