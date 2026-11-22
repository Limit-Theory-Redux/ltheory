-- Checks the navigation queries and the events of orders and zones:
--   * the neighbours of a ship (order, radius, the ship itself, a filter, a compound body, a body with
--     no entity) and the stall rule on its own;
--   * arrived: a point of a patrol, the end of a move, the join of an orbit;
--   * replanned: a change of target or of cost profile, a target that drifts, a zone whose cost changes;
--     and not a first plan, a resume, or a refresh by timer or after a corner, which happen quietly and
--     leave the flight in its phase;
--   * blocked: a ship that can't move, and not one that flies;
--   * a ship entering and leaving a zone, and which ships are watched.
-- The simulation is stepped by hand with `Nav.update`, not by frames; the events are checked in frames
-- after the flights, since the event bus delivers them there. Logs each failed check and ends with a
-- failure exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local SpatialEntities     = require("Modules.Spatial.Entities")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local Nav                 = require("Modules.Navigation")
local NavConfig           = require("Config.Navigation.NavConfig")
local CostProfileDefs     = require("Shared.Registries.CostProfileRegistry")
local ProgressWatch       = require("Shared.Helpers.ProgressWatch")
local PlanFlight          = require("Shared.Helpers.PlanFlight")
local FlightScene         = require("States.App.Tests.Support.FlightScene")

---@class NavigationEventsTest: Application
local EventsTest = Subclass("NavigationEventsTest", Application)

local Status     = Enums.NavOrderStatus
local Components = Nav.Components
local Phase      = PlanFlight.Phase

local STEP = 1 / 60
-- Bounds that turn a flight or a wait that never ends into a failure.
local MAX_FLIGHT_S = 240
local PLAN_WAIT_S  = 30
local MS_PER_S     = 1000
local MAX_FRAMES   = 600
-- Frames the events get to arrive in after the last flight.
local EVENT_FRAMES = 3

-- Distances, in game units. A point goal within the drive's minimum length flies without the drive.
local LEG            = 40
local SECONDS_BEFORE_CHANGE = 10
-- A flight that lasts longer than the stall window, to show that a ship that moves is not stalled.
local LONG_LEG       = 100
-- A stalled ship is watched this many windows, to show that a stall is reported once.
local STALL_WINDOWS  = 2.5
-- A target ship, farther than its stand-off and arrival distances, moves by less than the drift rule's
-- share of its distance, then by more.
local DRIFT_DISTANCE = LONG_LEG
local DRIFT_SMALL = DRIFT_DISTANCE * NavConfig.goalDriftShare / 4
local DRIFT_LARGE = DRIFT_DISTANCE / 2
-- The zone of the cost change is flown for this long before it is made dear, and is dear by this cost.
local SECONDS_BEFORE_ZONE_CHANGE = 2
local DEAR_COST = 50

-- The neighbours scene: the radius asked, and the bodies by their distance from the ship along an axis.
local NEIGHBOUR_RADIUS = 40
local BODY_RADIUS      = 2
local NEAR_AT, MID_AT, FAR_AT, OUT_AT = 10, 25, 30, 60
-- The child collider of the compound body: its offset in the parent's scale, so that it is nearer than the parent.
local CHILD_OFFSET     = -5
local NO_ENTITY_AT     = -15
local NEAR_DISTANCE    = 20

-- The zones: their radius, and where the through-flight's zone lies along the flight.
local ZONE_RADIUS = 8
local ZONE_AT     = LEG / 2
local START_ZONE_RADIUS = 5
-- The point flights of the patrol, along its legs.
local PATROL_POINTS = 3

-- Progress and window of the stall rule, for the rule's own checks.
local RULE_DISTANCE = 1000

function EventsTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationEventsTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---@param ship { rb: RigidBody }
---@param goal Position
---@param radius gameunit
---@return boolean
local function within(ship, goal, radius)
    return ship.rb:getPos():distance(goal) <= radius
end

---One step of the simulation, in the order a frame runs it. A ship that can't move gets no thrust.
---@param noThrust boolean|nil
function EventsTest:step(noThrust)
    Nav.update(STEP)
    if not noThrust then
        TravelDriveSystem:update(STEP)
        ShipFlightSystem:update()
    end
    self.world:update(STEP)
    self.simTime = self.simTime + STEP
end

---Steps until `done()` holds, for at most `seconds` of simulated time.
---@param done fun(): boolean
---@param seconds number
---@param noThrust boolean|nil
---@return boolean reached
function EventsTest:flyUntil(done, seconds, noThrust)
    for _ = 1, math.floor(seconds / STEP) do
        if done() then return true end
        self:step(noThrust)
    end
    return done()
end

---Updates with no time passing until `done()` holds; the ships don't move. The planner answers in its
---own time, so this waits on the work, for at most `PLAN_WAIT_S` of real time.
---@param done fun(): boolean
---@return boolean reached
function EventsTest:awaitWork(done)
    local start = InstantTime.Now()
    while not done() and InstantTime.Now():durationSince(start) < PLAN_WAIT_S * MS_PER_S do
        Nav.update(0)
    end
    return done()
end

---Updates with no time passing until the order's plan has come back; the ship doesn't move.
---@param order Entity
function EventsTest:awaitPlan(order)
    self:awaitWork(function() return Nav.status(order) ~= Status.Planning end)
end

---Updates with no time passing until the order has taken `count` refreshed plans.
---@param flight OrderFlightComponent
---@param count integer
---@param name string
function EventsTest:awaitAdopted(flight, count, name)
    self:check(self:awaitWork(function() return flight.adopted >= count end), name .. "the new plan takes over")
end

---@param at Position
---@return { entity: Entity, rb: RigidBody }
function EventsTest:ship(at)
    local ship = FlightScene.spawnShip(self.world, at)
    self.world:update(STEP)
    return ship
end

---Ends a case: releases its orders (they have ended), and removes its ships and bodies.
---@param orders Entity[]
---@param things { entity: Entity, rb: RigidBody }[]
function EventsTest:finish(orders, things)
    for _, order in ipairs(orders) do
        local status = Nav.status(order)
        self:check(status == Status.Done or status == Status.Cancelled, "an order that is released has ended")
        Nav.release(order)
    end
    FlightScene.despawn(self.world, things)
    -- The query index of the next case starts without them.
    self.world:update(STEP)
end

---Flies a ship to its order's end.
---@param order Entity
---@param name string
---@return boolean
function EventsTest:fly(order, name)
    Nav.update(0)
    self:awaitPlan(order)
    local reached = self:flyUntil(function() return Nav.status(order) == Status.Done end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done",
        string.format("status %s, %s left", tostring(Nav.status(order)), tostring(Nav.progress(order).distanceLeft)))
    return reached
end

---Remembers how many events of `name` the order is to be announced with.
---@param name string
---@param order Entity
---@param count integer
function EventsTest:expect(name, order, count)
    self.expected[name][order.id] = count
end

--- The stall rule on its own: a stall is reported once, progress re-arms it, and a step smaller than the
--- rule's progress is none.
function EventsTest:caseProgressWatch()
    local name = "the stall rule: "
    local window, step = NavConfig.blockedWindowS, NavConfig.blockedProgress()
    local watch = ProgressWatch.new(0)
    self:check(not ProgressWatch.update(watch, RULE_DISTANCE, 0), name .. "the first distance is progress")
    self:check(not ProgressWatch.update(watch, RULE_DISTANCE, window / 2), name .. "no stall inside the window")
    self:check(ProgressWatch.update(watch, RULE_DISTANCE, window), name .. "a stall at the end of the window")
    self:check(not ProgressWatch.update(watch, RULE_DISTANCE, window * 2), name .. "a stall is reported once")

    self:check(not ProgressWatch.update(watch, RULE_DISTANCE - step, window * 2), name .. "progress re-arms the watch")
    self:check(not ProgressWatch.update(watch, RULE_DISTANCE - step, window * 3 - 1), name .. "and restarts the window")
    self:check(ProgressWatch.update(watch, RULE_DISTANCE - step, window * 3), name .. "and a new stall is reported")

    ProgressWatch.reset(watch, window * 4)
    self:check(not ProgressWatch.update(watch, RULE_DISTANCE, window * 4), name .. "a reset starts over")
    self:check(ProgressWatch.update(watch, RULE_DISTANCE - step / 2, window * 5),
        name .. "less than the rule's progress is none")
end

---The neighbours of a ship: in range, nearest first, each entity once at its nearest collider, the
---distance to the surface, a filter, and the ship itself and a body with no entity left out.
function EventsTest:caseNeighbours()
    local name = "neighbours: "
    local ship = self:ship(Position(0, 0, 0))
    local near = FlightScene.spawnBody(self.world, Position(NEAR_AT, 0, 0), BODY_RADIUS)
    local far = FlightScene.spawnBody(self.world, Position(FAR_AT, 0, 0), BODY_RADIUS)
    local out = FlightScene.spawnBody(self.world, Position(OUT_AT, 0, 0), BODY_RADIUS)
    -- A compound: its collider that is nearer lists it again, with the entity's id.
    local compound = FlightScene.spawnBody(self.world, Position(0, MID_AT, 0), BODY_RADIUS)
    local child = RigidBody.CreateSphere()
    child:setScale(BODY_RADIUS)
    child:setUserId(compound.entity.id)
    compound.rb:attach(child, Vec3f(0, CHILD_OFFSET, 0), Quat.Identity())
    -- A body with no entity is listed by the query and skipped by the neighbours.
    local anonymous = RigidBody.CreateSphere()
    anonymous:setKinematic(true)
    anonymous:setScale(BODY_RADIUS)
    self.world:addRigidBody(anonymous)
    anonymous:setPos(Position(NO_ENTITY_AT, 0, 0))
    self.world:update(STEP)

    local found = Nav.neighbours(ship.entity, NEIGHBOUR_RADIUS)
    local ids = {}
    for i, neighbour in ipairs(found) do ids[i] = neighbour.entity.id end
    self:check(#found == 3 and ids[1] == near.entity.id and ids[2] == compound.entity.id and ids[3] == far.entity.id,
        name .. "the entities in range, nearest first, each once", table.concat(ids, ", "))
    for i = 2, #found do
        self:check(found[i - 1].distance <= found[i].distance, name .. "the distances don't fall")
    end
    self:check(found[1] and math.abs(found[1].distance - (NEAR_AT - BODY_RADIUS)) < 1e-6,
        name .. "the distance is to the surface", tostring(found[1] and found[1].distance))
    local viaChild = found[2] and found[2].distance or math.huge
    self:check(viaChild < MID_AT - BODY_RADIUS, name .. "a compound is at its nearest collider", tostring(viaChild))
    for _, neighbour in ipairs(found) do
        self:check(neighbour.entity.id ~= ship.entity.id, name .. "the ship is not its own neighbour")
        self:check(neighbour.entity.id ~= out.entity.id, name .. "a body out of range is not listed")
    end

    local filtered = Nav.neighbours(ship.entity, NEIGHBOUR_RADIUS, function(entity) return entity.id ~= near.entity.id end)
    self:check(#filtered == 2 and filtered[1].entity.id == compound.entity.id, name .. "a filter takes entities out")
    local closeOnly = Nav.neighbours(ship.entity, NEIGHBOUR_RADIUS, function(_, distance) return distance < NEAR_DISTANCE end)
    self:check(#closeOnly == 2 and closeOnly[2].entity.id == compound.entity.id, name .. "a filter sees the distance")
    self:check(#Nav.neighbours(ship.entity, BODY_RADIUS) == 0, name .. "a radius with nothing in it gives none")
    local bodiless = Registry:createEntity()
    self:check(not pcall(Nav.neighbours, bodiless, NEIGHBOUR_RADIUS), name .. "an entity with no body can't ask")
    Registry:destroyEntity(bodiless)

    self.world:removeRigidBody(anonymous)
    FlightScene.despawn(self.world, { near, far, out, compound })
    self:finish({}, { ship })
end

---A MoveTo sends one `NavOrderArrived` at its goal.
function EventsTest:caseMoveTo()
    local name = "a move: "
    local ship = self:ship(Position(0, 0, 0))
    local goal = Position(LEG, 0, 0)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = goal })
    self:fly(order, name)
    self:check(within(ship, goal, NavConfig.pointArrival() * 2), name .. "the ship is at the goal")
    self:expect("NavOrderArrived", order, 1)
    self:finish({ order }, { ship })
end

---A Patrol sends a `NavOrderArrived` at each of its points.
function EventsTest:casePatrol()
    local name = "a patrol: "
    local ship = self:ship(Position(0, 0, 0))
    local half = LEG / 2
    local points = { Position(half, 0, 0), Position(half, half, 0), Position(0, half, 0) }
    local order = Nav.order(ship.entity, Nav.Orders.Patrol { points = points })
    self:fly(order, name)
    self:expect("NavOrderArrived", order, PATROL_POINTS)
    self:finish({ order }, { ship })
end

---An Orbit sends a `NavOrderArrived` when the ship joins the circle.
function EventsTest:caseOrbit()
    local name = "an orbit: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(LEG, LEG / 4, 0), BODY_RADIUS)
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.Orbit { target = body.entity, altitude = BODY_RADIUS })
    Nav.update(0)
    self:awaitPlan(order)
    local flight = order:get(Components.OrderFlight)
    self:check(self:flyUntil(function() return flight.tracking end, MAX_FLIGHT_S), name .. "the ship joins the circle")
    self:expect("NavOrderArrived", order, 1)
    Nav.cancel(order)
    self:step()
    self:finish({ order }, { ship, body })
end

---A change of target in flight sends one `NavOrderReplanned`, and the order arrives.
function EventsTest:caseRetarget()
    local name = "a change of target: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE)
    Nav.retarget(order, Position(0, 0, -LEG))
    self:step()
    self:fly(order, name)
    self:expect("NavOrderArrived", order, 1)
    self:expect("NavOrderReplanned", order, 1)
    self:finish({ order }, { ship })
end

---A change of the ship's cost profile in flight sends one `NavOrderReplanned`, and the order arrives.
function EventsTest:caseCostProfileChange()
    local name = "a change of cost profile: "
    local ship = self:ship(Position(0, 0, 0))
    local profile = ship.entity:add(Components.CostProfile(CostProfileDefs:get(Enums.CostProfile.Standard)))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE)
    profile:setProfile(CostProfileDefs:get(Enums.CostProfile.Cautious))
    self:step()
    self:awaitAdopted(order:get(Components.OrderFlight), 1, name)
    self:fly(order, name)
    self:expect("NavOrderArrived", order, 1)
    self:expect("NavOrderReplanned", order, 1)
    self:finish({ order }, { ship })
end

---@param at Position
---@param radius gameunit
---@return Entity
local function sphereZone(at, radius)
    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    zone:get(SpatialComponents.Shape):setRadius(radius)
    zone:get(PhysicsComponents.Transform):setPos(at)
    return zone
end

---A target that no plan rides on (a ship) has drifted when it has moved more than a share of the ship's
---distance to it: a smaller move keeps the plan, a larger one makes a new plan at once, announced.
function EventsTest:caseTargetDrift()
    local name = "a drifting target: "
    local ship = self:ship(Position(0, 0, 0))
    local target = self:ship(Position(DRIFT_DISTANCE, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = target.entity })
    Nav.update(0)
    self:awaitPlan(order)
    local flight = order:get(Components.OrderFlight)
    local plan = flight.plan
    target.rb:setPos(Position(DRIFT_DISTANCE, DRIFT_SMALL, 0))
    self:step()
    self:check(flight.plan == plan, name .. "a small move keeps the plan")
    target.rb:setPos(Position(DRIFT_DISTANCE, DRIFT_LARGE, 0))
    self:step()
    self:check(flight.plan ~= plan, name .. "a large move makes a new plan")
    self:expect("NavOrderReplanned", order, 1)
    self:fly(order, name)
    self:expect("NavOrderArrived", order, 1)
    self:finish({ order }, { ship, target })
end

---A zone whose cost changes is a change of what the plan was made under: the ship keeps flying its plan
---while a new one is asked for, and the new one's taking over is announced.
function EventsTest:caseZoneCostChange()
    local name = "a zone made dear: "
    local ship = self:ship(Position(0, 0, 0))
    local zone = sphereZone(Position(ZONE_AT, 0, 0), ZONE_RADIUS)
    local cost = zone:add(Components.PathCost(1))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    local flight = order:get(Components.OrderFlight)
    self:flyUntil(function() return false end, SECONDS_BEFORE_ZONE_CHANGE)
    cost:setCost(DEAR_COST)
    self:step()
    self:check(flight.plan.phase ~= Phase.Planning, name .. "the ship flies its plan while the new one is made")
    self:awaitAdopted(flight, 1, name)
    self:expect("NavOrderReplanned", order, 1)
    self:fly(order, name)
    self:expect("NavOrderArrived", order, 1)
    Registry:destroyEntity(zone)
    self:finish({ order }, { ship })
end

---The timer asks for a fresh plan while a ship flies one: the ship is never left with no plan, and the
---plans that take over are not announced.
function EventsTest:caseQuietRefresh()
    local name = "refreshes of a plan: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LONG_LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    local flight = order:get(Components.OrderFlight)
    local interrupted = false
    local reached = self:flyUntil(function()
        if flight.plan and flight.plan.phase == Phase.Planning then interrupted = true end
        -- A refresh is asked of the worker, which answers in its own time: the flight waits for it.
        if flight.pending then self:awaitWork(function() return flight.pending == nil end) end
        return Nav.status(order) == Status.Done
    end, MAX_FLIGHT_S)
    self:check(reached, name .. "the order is done")
    self:check(not interrupted, name .. "the flight never goes back to planning")
    self:check(flight.adopted >= 1, name .. "a refreshed plan took over", tostring(flight.adopted))
    self:expect("NavOrderArrived", order, 1)
    self:finish({ order }, { ship })
end

---A routine order is suspended by an urgent one and resumes: its plan is dropped and planned again, which
---is not a change of the path.
function EventsTest:caseResume()
    local name = "a resume: "
    local ship = self:ship(Position(0, 0, 0))
    local routine = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(routine)
    self:flyUntil(function() return false end, SECONDS_BEFORE_CHANGE)
    local urgent = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(0, LEG, 0), priority = Nav.Priority.Urgent })
    self:step()
    self:check(Nav.status(routine) == Status.Suspended, name .. "the routine order is suspended")
    self:fly(urgent, name .. "the urgent order: ")
    self:fly(routine, name)
    self:expect("NavOrderArrived", urgent, 1)
    self:expect("NavOrderArrived", routine, 1)
    self:finish({ urgent, routine }, { ship })
end

---A ship that is given no thrust stays where it is, and is reported blocked once.
function EventsTest:caseBlocked()
    local name = "a ship that can't move: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    Nav.update(0)
    self:awaitPlan(order)
    self:flyUntil(function() return false end, NavConfig.blockedWindowS * STALL_WINDOWS, true)
    self:check(Nav.status(order) == Status.Active, name .. "the order goes on")
    self:expect("NavOrderBlocked", order, 1)
    -- The stall asks for a new plan, and its replacement is announced once it takes over.
    local flight = order:get(Components.OrderFlight)
    self:check(self:awaitWork(function() return flight.pending == nil and not flight.replanDue end),
        name .. "the refresh the stall asked for has come")
    self:expect("NavOrderReplanned", order, 1)
    Nav.cancel(order)
    self:step()
    self:finish({ order }, { ship })
end

---A flight that is longer than the stall window, flown by a ship that moves, is not a stall.
function EventsTest:caseLongFlight()
    local name = "a long flight: "
    local ship = self:ship(Position(0, 0, 0))
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LONG_LEG, 0, 0) })
    local start = self.simTime
    self:fly(order, name)
    self:check(self.simTime - start > NavConfig.blockedWindowS, name .. "it lasts longer than the stall window",
        string.format("%.1f s", self.simTime - start))
    self:expect("NavOrderArrived", order, 1)
    self:finish({ order }, { ship })
end

---Remembers the zone events a ship is to be announced with, in order, for one zone.
---@param ship { entity: Entity }
---@param zone Entity
---@param ... string "Entered" or "Left"
function EventsTest:expectZone(ship, zone, ...)
    self.expectedZone[ship.entity.id .. ":" .. zone.id] = table.concat({ ... }, ",")
end

---A flight through zones enters and leaves the one it crosses and the one it starts in, and not the one
---aside.
function EventsTest:caseZoneFlight()
    local name = "a flight through zones: "
    local ship = self:ship(Position(0, 0, 0))
    local across = sphereZone(Position(ZONE_AT, 0, 0), ZONE_RADIUS)
    local atStart = sphereZone(Position(0, 0, 0), START_ZONE_RADIUS)
    local aside = sphereZone(Position(ZONE_AT, LEG, 0), ZONE_RADIUS)
    local order = Nav.order(ship.entity, Nav.Orders.MoveTo { target = Position(LEG, 0, 0) })
    self:fly(order, name)
    self:expectZone(ship, across, "Entered", "Left")
    self:expectZone(ship, atStart, "Entered", "Left")
    self:expect("NavOrderArrived", order, 1)
    for _, zone in ipairs({ across, atStart, aside }) do Registry:destroyEntity(zone) end
    self:finish({ order }, { ship })
end

---A zone that is destroyed with the ship inside is left; an order that ends is no longer watched, with no
---event, and a new order announces the zone the ship is in again; a ship with no order is not watched.
function EventsTest:caseZoneWatch()
    local name = "the zone watch: "
    local ship = self:ship(Position(0, 0, 0))
    local bystander = self:ship(Position(LEG, LEG, 0))
    local destroyed = sphereZone(Position(0, 0, 0), START_ZONE_RADIUS)
    local hold = Nav.order(ship.entity, Nav.Orders.Hold())
    self:step()
    self:check(ship.entity:get(Components.ZoneMembership):isInside(destroyed.id), name .. "the ship is inside")
    Registry:destroyEntity(destroyed)
    self:step()
    self:expectZone(ship, destroyed, "Entered", "Left")

    local kept = sphereZone(Position(0, 0, 0), START_ZONE_RADIUS)
    local alone = sphereZone(Position(LEG, LEG, 0), START_ZONE_RADIUS)
    self:step()
    Nav.cancel(hold)
    self:step()
    self:step()
    self:check(ship.entity:get(Components.ZoneMembership) == nil, name .. "a ship with no order is not watched")
    local again = Nav.order(ship.entity, Nav.Orders.Hold())
    self:step()
    self:expectZone(ship, kept, "Entered", "Entered")
    self:check(bystander.entity:get(Components.ZoneMembership) == nil, name .. "a ship that never had an order is not watched")
    self:expectZone(bystander, alone)

    Nav.cancel(again)
    self:step()
    for _, zone in ipairs({ kept, alone }) do Registry:destroyEntity(zone) end
    self:finish({ hold, again }, { ship, bystander })
end

---Counts the events as they arrive, in the order they come.
function EventsTest:listen()
    self.events = {}
    local orderEvents = { "NavOrderArrived", "NavOrderReplanned", "NavOrderBlocked", "NavOrderDone" }
    for _, name in ipairs(orderEvents) do
        EventBus:subscribe(Event[name], nil, function(_, _, id)
            self.events[#self.events + 1] = { name = name, order = id }
        end)
    end
    for _, name in ipairs({ "NavZoneEntered", "NavZoneLeft" }) do
        EventBus:subscribe(Event[name], nil, function(_, _, payload)
            self.events[#self.events + 1] = { name = name, ship = payload.ship, zone = payload.zone }
        end)
    end
end

---The index of the `nth` event of a kind for an order, or nil.
---@param name string
---@param id EntityId
---@param nth integer
---@return integer|nil
function EventsTest:indexOf(name, id, nth)
    local seen = 0
    for i, event in ipairs(self.events) do
        if event.name == name and event.order == id then
            seen = seen + 1
            if seen == nth then return i end
        end
    end
    return nil
end

function EventsTest:checkOrderEvents()
    local tally = {}
    for _, event in ipairs(self.events) do
        if event.order then
            tally[event.name] = tally[event.name] or {}
            tally[event.name][event.order] = (tally[event.name][event.order] or 0) + 1
        end
    end
    for name, expected in pairs(self.expected) do
        local received = tally[name] or {}
        for id, count in pairs(expected) do
            self:check(received[id] == count, string.format("order %d gets %d %s", id, count, name),
                tostring(received[id]))
        end
        for id in pairs(received) do
            self:check(expected[id] ~= nil, string.format("order %d gets %s only when it should", id, name))
        end
    end
    -- A move is announced as arrived before it is done.
    for id in pairs(self.expected.NavOrderArrived) do
        local done = tally.NavOrderDone and tally.NavOrderDone[id]
        if done then
            local last = self.expected.NavOrderArrived[id]
            local arrived, finished = self:indexOf("NavOrderArrived", id, last), self:indexOf("NavOrderDone", id, 1)
            self:check(arrived and finished and arrived < finished, "order " .. id .. " arrives before it is done")
        end
    end
end

function EventsTest:checkZoneEvents()
    local received = {}
    for _, event in ipairs(self.events) do
        if event.zone then
            local key = event.ship .. ":" .. event.zone
            local kind = event.name == "NavZoneEntered" and "Entered" or "Left"
            received[key] = received[key] and (received[key] .. "," .. kind) or kind
        end
    end
    for key, expected in pairs(self.expectedZone) do
        self:check((received[key] or "") == expected, "the zone events of " .. key, tostring(received[key]))
    end
    for key in pairs(received) do
        self:check(self.expectedZone[key] ~= nil, "no zone event for " .. key .. " when none is due")
    end
end

function EventsTest:checkEvents()
    self:checkOrderEvents()
    self:checkZoneEvents()
    self:check(#self.events > 0, "events arrived")
end

function EventsTest:onInit()
    self.passed, self.failed = 0, 0
    self.simTime = 0
    self.frames = 0
    self.expected = { NavOrderArrived = {}, NavOrderReplanned = {}, NavOrderBlocked = {} }
    self.expectedZone = {}

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    Nav.attach(self.world)
    self:listen()

    self:caseProgressWatch()
    self:caseNeighbours()
    self:caseMoveTo()
    self:casePatrol()
    self:caseOrbit()
    self:caseRetarget()
    self:caseCostProfileChange()
    self:caseTargetDrift()
    self:caseZoneCostChange()
    self:caseQuietRefresh()
    self:caseResume()
    self:caseBlocked()
    self:caseLongFlight()
    self:caseZoneFlight()
    self:caseZoneWatch()

    -- The events are delivered in the frames that follow; no order runs in them.
    EventBus:subscribe(Event.PostInput, self, function()
        self.frames = self.frames + 1
        if self.frames < EVENT_FRAMES and self.frames < MAX_FRAMES then return end
        self:checkEvents()
        Log.Info("NavigationEventsTest: %d checks passed, %d failed", self.passed, self.failed)
        self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
    end)
end

return EventsTest
