-- Shows how a game uses navigation, in the frame loop: the simulation is stepped by frames, never by
-- `Nav.update`, orders are given and read between frames, and an issuer reacts to the events that
-- arrive at PostSim. The cases read as the use cases of doc/script/nav/guide.md:
--   * a move to a body, the order released when `NavOrderDone` arrives, progress and route read each frame;
--   * a destination chosen with `Nav.estimateMany`, the next one ordered when an order is done;
--   * a higher-priority order that suspends the running one, which resumes and is done;
--   * a detach that cancels the orders that have not ended.
-- The case code is a coroutine that waits for the work itself (an event, a status). Logs each failed
-- check and ends with a failure exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local Nav                 = require("Modules.Navigation")
local NavConfig           = require("Config.Navigation.NavConfig")
local PlanFlight          = require("Shared.Helpers.PlanFlight")
local ApproachSpec        = require("Shared.Types.ApproachSpec")
local FlightScene         = require("States.App.Tests.Support.FlightScene")

---@class NavigationUsageTest: Application
local UsageTest = Subclass("NavigationUsageTest", Application)

local Status = Enums.NavOrderStatus

-- A wait that never ends becomes a failure after this many seconds of simulated time.
local MAX_WAIT_S = 120
-- Simulated seconds per second of the clock, as the game's time acceleration: the flights take seconds
-- of simulated time that a run does not wait for in real time.
local TIME_SCALE = 8
-- The longest simulated step: a hitch of the first frames would otherwise be one huge physics step.
local MAX_STEP_S = TIME_SCALE / 20

-- Distances, in game units. A point within the drive's minimum length is flown without the drive.
local LEG         = 40
local BODY_RADIUS = 1
-- The gap from the body's hull a stop is ordered at, in metres.
local GAP_M       = 100000
-- A ship that has arrived is within this many arrival radii of its goal.
local ARRIVED_SLACK = 2
-- A ship counts as moving above this speed, in game units per second.
local MOVING = 0.5

local EVENT_NAMES = { "NavOrderDone", "NavOrderSuspended", "NavOrderResumed", "NavOrderCancelled" }

function UsageTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationUsageTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---Waits, a frame at a time, until `done()` holds.
---@param done fun(): boolean
---@param what string Named in the failure when the wait runs out
---@return boolean reached
function UsageTest:waitUntil(done, what)
    local start = self.simSeconds
    while not done() do
        if self.simSeconds - start > MAX_WAIT_S then
            self:check(false, what, "ran out of time")
            return false
        end
        coroutine.yield()
    end
    return true
end

---Waits for one frame, which steps the physics: a body that has just been made is in the query index
---that sensing and the planner read only after it.
function UsageTest:nextFrame()
    coroutine.yield()
end

---Runs `onDone` when the order's `NavOrderDone` arrives (an issuer compares the event's payload with the
---ids of its own orders).
---@param order Entity
---@param onDone fun()
function UsageTest:whenDone(order, onDone)
    self.doneHandlers[order.id] = onDone
end

---The order is released when it is done, as an issuer does in its `NavOrderDone` handler.
---@param order Entity
function UsageTest:releaseWhenDone(order)
    self:whenDone(order, function() Nav.release(order) end)
end

---@param order Entity
---@return boolean
local function isReleased(order)
    return Registry:getEntityById(order.id) == nil
end

---The position in the event log of an order's event of a kind, or nil.
---@param name string
---@param order Entity
---@return integer|nil
function UsageTest:indexOf(name, order)
    for i, event in ipairs(self.events) do
        if event.name == name and event.order == order.id then return i end
    end
    return nil
end

---@param at Position
---@return { entity: Entity, rb: RigidBody }
function UsageTest:ship(at)
    local ship = FlightScene.spawnShip(self.world, at)
    self:nextFrame()
    return ship
end

---@param things { entity: Entity, rb: RigidBody }[]
function UsageTest:despawn(things)
    FlightScene.despawn(self.world, things)
    self:nextFrame()
end

function UsageTest:caseMoveToBody()
    local name = "a move to a body: "
    local ship = self:ship(Position(0, 0, 0))
    local bodyPos = Position(LEG, LEG / 4, -LEG / 2)
    local body = FlightScene.spawnBody(self.world, bodyPos, BODY_RADIUS)
    self:nextFrame()

    local approach = ApproachSpec.Of { kind = Enums.ApproachKind.StandOff, gapM = GAP_M }
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = body.entity, approach = approach })
    self:releaseWhenDone(order)

    -- Read each frame, into the table the caller keeps.
    local route = {}
    local firstLeft, lastLeft, sawRoute = nil, nil, false
    local reached = self:waitUntil(function()
        if isReleased(order) then return true end
        local left = Nav.progress(order).distanceLeft
        if left then
            firstLeft = firstLeft or left
            lastLeft = left
        end
        sawRoute = sawRoute or #Nav.route(order, route) > 0
        return false
    end, name .. "the order is done")
    self:nextFrame()

    self:check(reached and isReleased(order), name .. "the order is released when its event arrives")
    self:check(self:indexOf("NavOrderDone", order) ~= nil, name .. "NavOrderDone arrived")
    self:check(firstLeft ~= nil and lastLeft < firstLeft, name .. "the distance left shrinks",
        string.format("%s then %s", tostring(firstLeft), tostring(lastLeft)))
    self:check(sawRoute, name .. "the route lists the path left to fly")
    local gap = approach:getGap()
    local dist = ship.rb:getPos():distance(bodyPos)
    local arrival = PlanFlight.arrivalRadius(BODY_RADIUS, gap)
    self:check(math.abs(dist - PlanFlight.standOff(BODY_RADIUS, gap)) <= arrival * ARRIVED_SLACK,
        name .. "the ship stops at the stand-off", string.format("%.3f", dist))
    self:despawn({ ship, body })
end

---The index of the reachable estimate with the least time, or nil when none is reachable.
---@param estimates NavEstimateResult[]
---@return integer|nil
local function cheapest(estimates)
    local best = nil
    for i, estimate in ipairs(estimates) do
        if estimate.reachable and (not best or estimate.time < estimates[best].time) then best = i end
    end
    return best
end

function UsageTest:caseChooseDestination()
    local name = "a chosen destination: "
    local ship = self:ship(Position(0, 0, 0))
    local gone = FlightScene.spawnBody(self.world, Position(0, LEG, 0), BODY_RADIUS)
    local near, far = Position(LEG, 0, 0), Position(0, 0, 2 * LEG)
    local candidates = { far, gone.entity, near }
    self:despawn({ gone })

    local visited = {}
    -- The time the first scoring gave each point.
    local firstTime = {}
    -- The loop of an AI that scores its candidates, flies to the best, and scores the rest again.
    while true do
        local request = Nav.estimateMany(ship.entity, candidates)
        local estimates
        self:waitUntil(function()
            estimates = Nav.takeEstimates(request)
            return estimates ~= nil
        end, name .. "the estimates come back")
        if estimates and #visited == 0 then
            firstTime[candidates[1]], firstTime[candidates[3]] = estimates[1].time, estimates[3].time
        end
        local best = estimates and cheapest(estimates)
        if not best then break end

        visited[#visited + 1] = candidates[best]
        local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = candidates[best] })
        self:releaseWhenDone(order)
        self:waitUntil(function() return isReleased(order) end, name .. "the order is done")
        table.remove(candidates, best)
    end

    self:check(#visited == 2 and visited[1] ~= visited[2] and firstTime[visited[1]] <= firstTime[visited[2]],
        name .. "both points are flown to, the one with the least estimated time first", tostring(#visited))
    self:check(#candidates == 1, name .. "the target that is gone is never ordered")
    self:check(#visited == 2 and ship.rb:getPos():distance(visited[2]) <= NavConfig.pointArrival() * ARRIVED_SLACK,
        name .. "the ship is at the last point")
    self:despawn({ ship })
end

function UsageTest:caseInterrupt()
    local name = "a higher priority: "
    local ship = self:ship(Position(0, 0, 0))
    local goal, detour = Position(LEG, 0, 0), Position(0, LEG / 2, 0)
    local routine = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal })
    self:releaseWhenDone(routine)
    self:waitUntil(function() return ship.rb:getSpeed() > MOVING end, name .. "the first order is under way")

    local urgent = Nav.order(ship.entity, Nav.Orders.MoveTo { target = detour, priority = Nav.Priority.Urgent })
    self:releaseWhenDone(urgent)
    self:waitUntil(function() return isReleased(routine) and isReleased(urgent) end, name .. "both orders are done")
    self:nextFrame()

    local suspended = self:indexOf("NavOrderSuspended", routine)
    local urgentDone = self:indexOf("NavOrderDone", urgent)
    local resumed = self:indexOf("NavOrderResumed", routine)
    local routineDone = self:indexOf("NavOrderDone", routine)
    self:check(suspended and urgentDone and resumed and routineDone
        and suspended < urgentDone and urgentDone < resumed and resumed < routineDone,
        name .. "the first order is suspended, resumes once the urgent one is done, and is done",
        string.format("%s %s %s %s", tostring(suspended), tostring(urgentDone), tostring(resumed),
            tostring(routineDone)))
    self:check(ship.rb:getPos():distance(goal) <= NavConfig.pointArrival() * ARRIVED_SLACK,
        name .. "the ship is at the first order's point")
    self:despawn({ ship })
end

function UsageTest:caseDetach()
    local name = "a detach: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(0, 0, 4 * LEG) })
    self:waitUntil(function() return ship.rb:getSpeed() > MOVING end, name .. "the order is under way")

    -- Before the ship's body leaves the physics world.
    Nav.detach()
    self:check(Nav.status(order) == Status.Cancelled, name .. "an order that has not ended is cancelled",
        tostring(Nav.status(order)))
    Nav.release(order)
    self:check(isReleased(order), name .. "a cancelled order can be released")
    self:despawn({ ship })
end

---Runs the cases one after the other; the frames of the app resume it.
function UsageTest:runCases()
    for _, case in ipairs({ "caseMoveToBody", "caseChooseDestination", "caseInterrupt", "caseDetach" }) do
        local start = self.simSeconds
        self[case](self)
        Log.Info("NavigationUsageTest: %s took %.1f s of simulated time", case, self.simSeconds - start)
    end
    Log.Info("NavigationUsageTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

function UsageTest:onInit()
    self.passed, self.failed = 0, 0
    self.simSeconds = 0
    self.events = {}
    self.doneHandlers = {}
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    EventBus:setTimeScale(TIME_SCALE)
    Nav.attach(self.world)
    -- After the world: the events exist once the navigation system does.
    for _, name in ipairs(EVENT_NAMES) do
        EventBus:subscribe(Event[name], nil, function(_, _, id)
            self.events[#self.events + 1] = { name = name, order = id }
            local handler = name == "NavOrderDone" and self.doneHandlers[id]
            if handler then handler() end
        end)
    end
    self.cases = coroutine.create(function() self:runCases() end)
end

---The frame's start: the case code runs here, after the events of the last frame have arrived.
---@param data EventData
function UsageTest:onPreSim(data)
    local ok, err = coroutine.resume(self.cases)
    if not ok then Log.Error("NavigationUsageTest: %s", tostring(err)) end
end

---Ships fly on the control input the orders wrote at PreSim.
---@param data EventData
function UsageTest:onSim(data)
    local dt = math.min(data:deltaTime(), MAX_STEP_S)
    self.simSeconds = self.simSeconds + dt
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.world:update(dt)
end

-- Nothing is drawn.
function UsageTest:onPreRender() end

function UsageTest:onRender() end

function UsageTest:onPostRender() end

return UsageTest
