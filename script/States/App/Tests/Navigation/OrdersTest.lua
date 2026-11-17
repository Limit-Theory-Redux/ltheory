-- Checks the navigation orders: MoveTo to a point past a planet (a plan with a corner, the drive) and
-- to a body, a queue, Patrol with and without a loop, Hold, cancel (a queued order and a running
-- one), a change of target and of cost profile in flight (the path is planned again), a target and
-- a ship that are destroyed, release, and the world's detach. The end events are checked in frames
-- after the flights, since the event bus delivers them there. Logs each failed check and ends
-- with a failure exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local OrbitalSystem       = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local Nav                 = require("Modules.Navigation")
local NavConfig           = require("Config.Navigation.NavConfig")
local CostProfileDefs     = require("Shared.Registries.CostProfileRegistry")
local PlanFlight          = require("Shared.Helpers.PlanFlight")
local FlightScene         = require("States.App.Tests.Support.FlightScene")

---@class NavigationOrdersTest: Application
local OrdersTest = Subclass("NavigationOrdersTest", Application)

local Status  = Enums.NavOrderStatus
local Failure = Enums.NavOrderFailure
local Phase   = PlanFlight.Phase
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

    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity, standOffM = GAP_M })
    local gap = Nav.Orders.MoveTo { target = body.entity, standOffM = GAP_M }.gap
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
    self:check(planOf(order) ~= before, name .. "the path is planned again")
    self:awaitPlan(order)
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:finish({ order }, { ship })
end

function OrdersTest:caseTargetDestroyed()
    local name = "a destroyed target: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG, 0, LEG / 2), BODY_RADIUS)
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity, standOffM = GAP_M })
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
    self:check(expectedCount > 0, "orders ended")
end

---Counts the events as they arrive, per kind of end.
function OrdersTest:listen()
    self.received = { [Status.Done] = {}, [Status.Failed] = {}, [Status.Cancelled] = {} }
    self.duplicates = 0
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
    self:caseQueue()
    self:casePatrol(false)
    self:casePatrol(true)
    self:caseHold()
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
