local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local Components          = require("Modules.Navigation.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local ControlOwnerSystem  = require("Modules.Constructs.Systems.ControlOwnerSystem")
local NavConfig           = require("Config.Navigation.NavConfig")
local EntityPosition      = require("Shared.Helpers.EntityPosition")
local NavAgentHelper      = require("Shared.Helpers.NavAgentHelper")
local PlanFlight          = require("Shared.Helpers.PlanFlight")
local OrderGoals          = require("Shared.Helpers.OrderGoals")
local OrderHandlers       = require("Shared.Helpers.OrderHandlers")
local ProgressWatch       = require("Shared.Helpers.ProgressWatch")
local CostZones           = require("Shared.Helpers.CostZones")
local NavZoneSystem       = require("Modules.Navigation.Systems.NavZoneSystem")
local NavRateSystem       = require("Modules.Navigation.Systems.NavRateSystem")
local Stagger             = require("Shared.Helpers.Stagger")
local NavSpeedCaps        = require("Shared.Helpers.NavSpeedCaps")

local Phase   = PlanFlight.Phase
local Status  = Enums.NavOrderStatus
local Failure = Enums.NavOrderFailure

-- The event each end of an order is announced with; the payload is the order's entity id.
local END_EVENTS = {
    [Status.Done]      = "NavOrderDone",
    [Status.Failed]    = "NavOrderFailed",
    [Status.Cancelled] = "NavOrderCancelled",
}

-- The events announced when a running order gives way to one of higher priority, and when it takes
-- the ship back; the payload is the order's entity id.
local TRANSITION_EVENTS = {
    suspended = "NavOrderSuspended",
    resumed   = "NavOrderResumed",
}

-- The events about a running order; the payload is the order's entity id. `NavOrderArrived`: the ship
-- reached the goal (each Patrol point; a MoveTo just before it is done; an Orbit or Follow taking its
-- circle or slot). `NavOrderReplanned`: a plan that was flown is made again. `NavOrderBlocked`: the
-- ship has not come closer to its goal for the stall window (once per stall).
local FLIGHT_EVENTS = {
    arrived   = "NavOrderArrived",
    replanned = "NavOrderReplanned",
    blocked   = "NavOrderBlocked",
}

---@class OrderHead A ship's head order, with what the frame needs of it
---@field order Entity
---@field ship Entity
---@field slot integer The ship's agent
---@field handler OrderHandler|nil Nil for a Hold
---@field goal OrderGoal|nil
---@field facing FacingGoalComponent|nil What the nose is to face, when the order says
---@field zoneSignature integer `CostZones.signature` of this frame
---@field drift integer|nil The handler's drift key of this frame; nil for a goal that does not move

local bodyOf = OrderGoals.bodyOf

---@param ship Entity
---@return integer The revision of the ship's cost profile, 0 for a ship without one
local function costProfileRevision(ship)
    local profile = ship:get(Components.CostProfile)
    return profile and profile:getRevision() or 0
end

---Whether orders own the ship's control input, so that the system may write it. A ship that the player
---pilots or that an AI has claimed is not flown by orders.
---@param ship Entity
---@return boolean
local function flownByOrders(ship)
    return ControlOwnerSystem:ownerOf(ship) == Enums.ControlOwner.Orders
end

---Takes a ship's agent out of the world. The control input is not touched: whoever owns it now
---(or, when the last order ends, the ownership system) resets it.
---@param world NavWorldComponent
---@param ship Entity
local function detachAgent(world, ship)
    local agent = ship:get(Components.NavAgent)
    if not agent then return end
    world:getNav():removeAgent(agent:getSlot())
    NavSpeedCaps.forget(agent:getSlot())
    world:unassign(agent:getSlot())
    ship:remove(Components.NavAgent)
    ship:remove(Components.NavRate)
end

---Tells the travel drive to stop, if orders own the ship's input.
---@param ship Entity
local function disengageDrive(ship)
    local input = ship:get(ConstructComponents.ControlInput)
    if input and flownByOrders(ship) then input:setDriveRequest(Enums.TravelDriveRequest.Disengage) end
end

---Gives a ship an agent in the world, if it has none.
---@param world NavWorldComponent
---@param ship Entity
---@param rb RigidBody
local function attachAgent(world, ship, rb)
    if ship:get(Components.NavAgent) then return end
    local nav = world:getNav()
    local slot = NavAgentHelper.addShip(nav, ship, rb)
    world:reserve(nav:slotCount())
    world:assign(slot, ship.id)
    ship:add(Components.NavAgent(slot))
    local maxSpeed = ship:get(Components.MaxSpeed)
    if maxSpeed then NavSpeedCaps.limit(nav, slot, maxSpeed:getSpeed()) end
end

---Drops the flight's plan and the refresh of it that is out; a reply still to come is thrown away.
---@param flight OrderFlightComponent
local function dropPlans(flight)
    PlanFlight.drop(flight.plan)
    flight.plan = nil
    if flight.pending then PlanFlight.drop(flight.pending.plan) end
    flight.pending = nil
    flight.replanDue = false
    flight.legsBefore = 0
end

---Ends an order: its status, its plan, its place in the ship's queue, the ship's agent if the queue
---is empty, and the event that announces it.
---@param world NavWorldComponent
---@param order Entity
---@param ship Entity|nil The ship, nil when it is gone
---@param status NavOrderStatusId `Done`, `Failed` or `Cancelled`
---@param failure NavOrderFailureId|nil
local function endOrder(world, order, ship, status, failure)
    local orderStatus = order:get(Components.OrderStatus)
    orderStatus:setStatus(status)
    if failure then orderStatus:setFailure(failure) end
    dropPlans(order:get(Components.OrderFlight))

    if ship then
        local queue = ship:get(Components.OrderQueue)
        queue:remove(order.id)
        disengageDrive(ship)
        if queue:count() == 0 then
            detachAgent(world, ship)
            -- With no order left, the ship's controls are reset if orders were the ones to own them.
            ControlOwnerSystem:setOrders(ship, false)
        end
    end
    EventBus:dispatch(Event[END_EVENTS[status]], order.id)
end

---Removes the agents whose ship is gone, before the world steps: it panics if an agent's body has left.
---@param world NavWorldComponent
local function dropLostAgents(world)
    for slot, shipId in world:slots() do
        if not Registry:getEntityById(shipId) then
            world:getNav():removeAgent(slot)
            NavSpeedCaps.forget(slot)
            world:unassign(slot)
        end
    end
end

---Ends the order if it is cancelled or its ship is lost. Returns the ship while the order goes on.
---@param world NavWorldComponent
---@param order Entity
---@param navOrder Component
---@return Entity|nil
local function checkOrder(world, order, navOrder)
    local ship = Registry:getEntityById(navOrder:getShipId())
    if order:get(Components.OrderStatus):isCancelRequested() then
        endOrder(world, order, ship, Status.Cancelled)
        return nil
    end
    if not ship or not bodyOf(ship) then
        endOrder(world, order, ship, Status.Failed, Failure.ShipLost)
        return nil
    end
    return ship
end

---Takes a running order off its ship, because an order of higher priority is the head now, or
---because the ship's controls belong to someone else: its plan is dropped and it will begin again
---from where the ship is. What the order itself holds (a Patrol's point, an Orbit's plane, its
---target and facing) stays.
---@param order Entity
---@param ship Entity
local function suspendOrder(order, ship)
    local flight = order:get(Components.OrderFlight)
    dropPlans(flight)
    flight.begun = false
    flight.tracking = false
    order:get(Components.OrderStatus):setStatus(Status.Suspended)
    disengageDrive(ship)
    EventBus:dispatch(Event[TRANSITION_EVENTS.suspended], order.id)
end

---Gives a suspended order the ship back, now that it is the head again.
---@param order Entity
local function resumeOrder(order)
    local status = order:get(Components.OrderStatus)
    if status:getStatus() ~= Status.Suspended then return end
    status:setStatus(Status.Planning)
    EventBus:dispatch(Event[TRANSITION_EVENTS.resumed], order.id)
end

---Starts a head order that has not begun: the ship gets its agent, and a Hold its goal.
---@param world NavWorldComponent
---@param head OrderHead
local function beginOrder(world, head)
    local flight = head.order:get(Components.OrderFlight)
    attachAgent(world, head.ship, bodyOf(head.ship))
    head.slot = head.ship:get(Components.NavAgent):getSlot()
    if flight.begun then return end
    flight.begun = true
    flight.elapsed = 0
    flight.watch = ProgressWatch.new(0)
    flight.watchedLeg = nil
    flight.advanced = false
    -- The agent outlasts an order, so a facing left by the order before is not this order's.
    world:getNav():clearFacing(head.slot)
    -- Nor is a detour held from it.
    world:getNav():refresh(head.slot)
    if not head.handler then
        world:getNav():setHold(head.slot, NavConfig.pointArrival())
        head.order:get(Components.OrderStatus):setStatus(Status.Active)
    end
end

---Points the nose of the order's ship at its facing goal, as of now. A facing entity that is gone
---ends the order as failed.
---@param world NavWorldComponent
---@param head OrderHead
local function aimFacing(world, head)
    local entityId = head.facing:getTargetEntityId()
    local point = head.facing:getTargetPos()
    if entityId then
        local target = Registry:getEntityById(entityId)
        if not target then
            endOrder(world, head.order, head.ship, Status.Failed, Failure.TargetLost)
            return
        end
        point = EntityPosition.Get(target)
    end
    world:getNav():setFacing(head.slot, point)
end

---@param ship Entity
---@return boolean Whether the ship has no travel drive, or its drive is neither charging nor engaged
local function driveIdle(ship)
    local drive = ship:get(ConstructComponents.TravelDrive)
    return not drive or drive:getState() == "idle"
end

---Asks the planner for the path to the order's goal from where the ship is.
---@param head OrderHead
---@return PlanFlightState
local function beginPlan(head)
    local goal = head.goal
    -- The travel drive thrusts along the nose, which a facing holds elsewhere: the path has no drive leg.
    return PlanFlight.begin(head.ship, bodyOf(head.ship), goal.targetPos, goal.safeRange,
        goal.arrivalRadius, goal.frameEntity, head.facing ~= nil)
end

---Notes what the plan being asked for is made under, and when the timer asks next. `delay` is the
---seconds until then.
---@param head OrderHead
---@param flight OrderFlightComponent
---@param delay number
local function notePlanned(head, flight, delay)
    flight.plannedProfileRevision = costProfileRevision(head.ship)
    flight.plannedZoneSignature = head.zoneSignature
    flight.plannedDrift = head.drift
    flight.nextRefresh = flight.elapsed + delay
    flight.replanDue = false
end

---Makes a new plan for the goal, dropping the one that was flown: the ship flies straight to the goal
---until it comes. A plan that was being flown and is made again is announced when `announce`.
---@param world NavWorldComponent
---@param head OrderHead
---@param key integer The goal's key
---@param announce boolean
local function startPlan(world, head, key, announce)
    local flight = head.order:get(Components.OrderFlight)
    -- An arc holds the agent to the corner's speed, which only the follower of that plan lifts.
    if PlanFlight.onArc(flight.plan) then NavSpeedCaps.cap(world:getNav(), head.slot, math.huge) end
    dropPlans(flight)
    flight.plan = beginPlan(head)
    -- The ship's course changes: the detour held for the old goal is not for this one.
    world:getNav():refresh(head.slot)
    flight.plannedGoalKey = key
    flight.refreshedLegs = 0
    -- The first refresh waits one period and a share of another, which keeps orders begun together apart.
    notePlanned(head, flight, NavRateSystem.replanPeriodS(head.ship) * (1 + Stagger.of(head.order.id)))
    -- The stall check measures a new plan from the start.
    flight.watchedLeg = nil
    if announce then EventBus:dispatch(Event[FLIGHT_EVENTS.replanned], head.order.id) end
end

---Asks for a new plan while the ship flies the old one, which the reply replaces (`adoptPending`).
---@param head OrderHead
---@param flight OrderFlightComponent
---@param announce boolean Whether the replacement is announced
local function requestRefresh(head, flight, announce)
    notePlanned(head, flight, NavRateSystem.replanPeriodS(head.ship))
    flight.refreshedLegs = flight.plan.legs
    local fresh = beginPlan(head)
    -- With the ship in range of the goal there is nothing to plan.
    if not fresh.taskId then return end
    flight.pending = { plan = fresh, announce = announce, legs = flight.plan.legs }
end

---Gives the order its plan: a new one when there is none or the goal is another one (the old is
---dropped), a refresh asked for in the background when what the plan was made under has changed, a
---goal that moves has gone far from where the plan ends, the ship has stalled, a corner has been passed,
---or the period has run out. A drift never drops a plan or a request that is out: a goal that moves
---fast would leave the ship with none. Replacing a plan that was being flown is announced for a goal
---change, a drift or what it was made under, but not for a corner or the period, unless the order went
---on to its next goal.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec Component
local function ensurePlan(world, head, spec)
    local flight = head.order:get(Components.OrderFlight)
    local handler, shipPos = head.handler, bodyOf(head.ship):getPos()
    local key = handler.key(spec, head.goal, shipPos, head.ship)
    head.drift = handler.drift and handler.drift(spec, head.goal, shipPos, head.ship)
    local advanced = flight.advanced
    flight.advanced = false
    local plan = flight.plan
    if not plan or flight.plannedGoalKey ~= key then
        startPlan(world, head, key, plan ~= nil and not advanced)
        return
    end
    local changed = flight.plannedProfileRevision ~= costProfileRevision(head.ship)
        or flight.plannedZoneSignature ~= head.zoneSignature
    -- The first plan is still out: it has nothing to fly meanwhile, so it is asked for again. A drift
    -- waits for the reply, which is then refreshed.
    if plan.phase == Phase.Planning then
        if changed then startPlan(world, head, key, true) end
        return
    end
    local pending = flight.pending
    if pending and changed then
        -- The request that is out was made under what has changed since.
        flight.replanDue = flight.replanDue or pending.announce
        PlanFlight.drop(pending.plan)
        flight.pending = nil
        pending = nil
    end
    if pending then
        if flight.replanDue then pending.announce, flight.replanDue = true, false end
        return
    end
    -- The planner times a path from a ship at rest, with the drive still to charge: a plan asked for under
    -- a drive run would place the corners' bodies late and cut the run short. Whatever is due waits.
    if PlanFlight.onArc(plan) or not driveIdle(head.ship) then return end
    local announce = changed or flight.plannedDrift ~= head.drift or flight.replanDue
    local corner = plan.phase == Phase.Plan and plan.legs > flight.refreshedLegs
    local period = plan.phase ~= Phase.Direct and flight.elapsed >= flight.nextRefresh
    if announce or corner or period then requestRefresh(head, flight, announce) end
end

---Takes the refresh that is out once it has come: it replaces the plan that is flown, if the ship is not
---on an arc and has passed no corner since it was asked for, which would leave the new plan's first
---corner behind. A reply with no usable plan is dropped, and the old plan is flown on.
---@param world NavWorldComponent
---@param head OrderHead
---@param flight OrderFlightComponent
local function adoptPending(world, head, flight)
    local pending = flight.pending
    if not pending then return end
    local fresh, old = pending.plan, flight.plan
    if not PlanFlight.receive(fresh) then return end
    if old.legs ~= pending.legs then
        flight.replanDue = flight.replanDue or pending.announce
        flight.pending = nil
        return
    end
    if PlanFlight.onArc(old) then return end
    flight.pending = nil
    PlanFlight.fly(fresh, {
        nav = world:getNav(), agent = head.slot, rb = bodyOf(head.ship),
        profile = ConstructComponents.FlightProfile.ProfileOf(head.ship),
    })
    if fresh.phase == Phase.Direct then return end
    flight.legsBefore = flight.legsBefore + old.legs
    flight.plan = fresh
    flight.refreshedLegs = 0
    flight.adopted = flight.adopted + 1
    if pending.announce then EventBus:dispatch(Event[FLIGHT_EVENTS.replanned], head.order.id) end
end

---Moves an order from flying its plan to keeping the world's behaviour: the plan is dropped, and the
---agent takes the behaviour, which the world flies from the next step on. Announces the arrival.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec Component
local function beginTracking(world, head, spec)
    local flight = head.order:get(Components.OrderFlight)
    dropPlans(flight)
    flight.tracking = true
    head.handler.track.start(world, head, spec)
    EventBus:dispatch(Event[FLIGHT_EVENTS.arrived], head.order.id)
end

---Switches a tracking order's flight: a follower far from its slot goes back to planning, and one near
---it (or a ship that has arrived) takes the behaviour. Returns whether the order is tracking now.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec Component
---@param flight OrderFlightComponent
---@return boolean
local function switchTracking(world, head, spec, flight)
    local track, goal = head.handler.track, head.goal
    local shipPos = bodyOf(head.ship):getPos()
    if flight.tracking and track.leaves(spec, goal, shipPos, head.ship) then flight.tracking = false end
    if not flight.tracking and track.joins(spec, goal, shipPos, head.ship) then beginTracking(world, head, spec) end
    return flight.tracking
end

---Before the step of a goal order: its plan, and its goal on the world unless the plan's corners are flown.
---A tracking order has neither: the world holds its behaviour.
---@param world NavWorldComponent
---@param head OrderHead
---@param dt number
local function aimGoal(world, head, dt)
    local spec = head.order:get(head.handler.spec)
    head.goal = head.handler.goal(spec)
    if not head.goal then
        endOrder(world, head.order, head.ship, Status.Failed, Failure.TargetLost)
        return
    end
    local flight = head.order:get(Components.OrderFlight)
    if head.handler.track and switchTracking(world, head, spec, flight) then return end
    ensurePlan(world, head, spec)
    adoptPending(world, head, flight)
    -- A plan that is still to come keeps its snapshot's clock; one taken now is advanced below.
    if flight.pending then flight.pending.plan.snapshot:advance(dt) end
    local plan, goal = flight.plan, head.goal
    local rb = bodyOf(head.ship)
    if plan.phase == Phase.Planning then
        PlanFlight.poll(plan, {
            nav = world:getNav(), agent = head.slot, rb = rb,
            profile = ConstructComponents.FlightProfile.ProfileOf(head.ship),
        })
    end
    flight.elapsed = flight.elapsed + dt
    if plan.snapshot then plan.snapshot:advance(dt) end
    if plan.phase == Phase.Plan then return end
    local shipPos = rb:getPos()
    local dist = shipPos:distance(goal.targetPos)
    PlanFlight.setGoal(world:getNav(), head.slot, goal.frameBody,
        PlanFlight.standOffGoal(shipPos, goal.targetPos, dist, dist, goal.safeRange), goal.arrivalRadius)
end

---After the step, with the plan's corners flown: moves the follower on and drives the legs.
---@param head OrderHead
---@param plan PlanFlightState
---@param flight OrderFlightComponent
---@param arrived boolean
---@param zoneMax number
---@return number eta Seconds
local function flyCorners(head, plan, flight, arrived, zoneMax)
    local ship = head.ship
    plan.follower:advance(arrived, ShipFlightSystem:getSpeed(ship), flight.elapsed, plan.listener,
        NavAgentHelper.isCarried(ship))
    local drive = ship:get(ConstructComponents.TravelDrive)
    local input = ship:get(ConstructComponents.ControlInput)
    if drive and input and plan.phase == Phase.Plan and not head.facing then
        PlanFlight.driveLeg(plan, drive, input, ConstructComponents.FlightProfile.ProfileOf(ship), zoneMax)
    end
    return math.max(0, plan.goal.time - plan.snapshot.age)
end

---After the step, on the way to the goal: the drive follows the braking law.
---@param head OrderHead
---@param plan PlanFlightState
---@param dist gameunit Distance to the target's centre
---@param zoneMax number
---@return number eta Seconds
local function flyApproach(head, plan, dist, zoneMax)
    local ship, goal = head.ship, head.goal
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship)
    local drive = ship:get(ConstructComponents.TravelDrive)
    local input = ship:get(ConstructComponents.ControlInput)
    -- The drive runs on the final approach and a flight with no usable plan, never while the plan is out
    -- or while the order has a facing.
    local driven = drive and input and plan.phase ~= Phase.Planning and not head.facing
    if driven then
        PlanFlight.updateDrive(drive, input, profile, zoneMax, dist, goal.safeRange)
    end
    return PlanFlight.travelTime(math.max(0, dist - goal.safeRange), goal.arrivalRadius, profile, zoneMax,
        not head.facing and PlanFlight.driveWillRun(drive, profile, math.max(0, dist - goal.safeRange)))
end

---Announces a stall of the flight: no progress for the stall window toward the leg being flown (the
---plan's corner, or the goal when there is no corner), which restarts the check at each leg.
---@param order Entity
---@param flight OrderFlightComponent
---@param plan PlanFlightState
---@param shipPos Position
---@param goalDist gameunit Distance to the goal, which is the measure when no corner is flown
local function checkBlocked(order, flight, plan, shipPos, goalDist)
    local atCorners = plan.phase == Phase.Plan
    -- The legs of the plans a refresh replaced count too: the check restarts at a corner, not at a refresh.
    local leg = atCorners and flight.legsBefore + plan.legs or -1
    if flight.watchedLeg ~= leg then
        flight.watchedLeg = leg
        ProgressWatch.reset(flight.watch, flight.elapsed)
    end
    local dist = atCorners and shipPos:distance(plan.follower.target) or goalDist
    if ProgressWatch.update(flight.watch, dist, flight.elapsed) then
        -- A stalled ship asks for a new plan, which may find the way round what holds it.
        flight.replanDue = true
        EventBus:dispatch(Event[FLIGHT_EVENTS.blocked], order.id)
    end
end

---After the step of a goal order: arrival, the corners or the approach, the axes, and the progress.
---@param world NavWorldComponent
---@param head OrderHead
local function finishGoal(world, head)
    local order, ship, goal = head.order, head.ship, head.goal
    local flight = order:get(Components.OrderFlight)
    local plan = flight.plan
    local rb = bodyOf(ship)
    local arrived = world:getArrived()[head.slot]
    local dist = rb:getPos():distance(goal.targetPos)
    local zoneMax = GravityWellSystem:getMaxDriveSpeedAt(rb:getPos())
    local atCorners = plan.phase == Phase.Plan

    if not atCorners and arrived then
        local spec = order:get(head.handler.spec)
        if head.handler.track then
            beginTracking(world, head, spec)
        else
            EventBus:dispatch(Event[FLIGHT_EVENTS.arrived], order.id)
            if head.handler.arrive(spec) then
                endOrder(world, order, ship, Status.Done)
            else
                flight.advanced = true
            end
        end
        return
    end
    local eta
    if atCorners then
        eta = flyCorners(head, plan, flight, arrived, zoneMax)
    else
        eta = flyApproach(head, plan, dist, zoneMax)
    end
    checkBlocked(order, flight, plan, rb:getPos(), dist)
    local input = ship:get(ConstructComponents.ControlInput)
    if input then NavAgentHelper.writeAxes(world:getAxes(), head.slot, input) end

    order:get(Components.OrderStatus):setStatus(plan.phase == Phase.Planning and Status.Planning or Status.Active)
    local progress = order:get(Components.OrderProgress)
    progress.distanceLeft = math.max(0, dist - goal.safeRange)
    progress.eta = eta
    -- The phase after the step: the follower that has just passed its last corner no longer steers to it.
    progress.nextWaypoint = plan.phase == Phase.Plan and plan.follower.target or goal.targetPos
    local patrol = order:get(Components.PatrolOrder)
    if patrol then progress.waypointIndex = patrol:getIndex() end
end

---After the step of a Hold: the axes, with the drive released.
---@param world NavWorldComponent
---@param head OrderHead
local function finishHold(world, head)
    local input = head.ship:get(ConstructComponents.ControlInput)
    if not input then return end
    NavAgentHelper.writeAxes(world:getAxes(), head.slot, input)
    local drive = head.ship:get(ConstructComponents.TravelDrive)
    if drive and drive:getState() ~= "idle" then
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
    end
end

---After the step of a tracking order: the axes of the world's behaviour, with the drive released, and
---the distance from the circle or the slot as the progress.
---@param world NavWorldComponent
---@param head OrderHead
local function finishTrack(world, head)
    finishHold(world, head)
    local goal = head.goal
    local progress = head.order:get(Components.OrderProgress)
    progress.distanceLeft = math.abs(bodyOf(head.ship):getPos():distance(goal.targetPos) - goal.safeRange)
    progress.eta = 0
    progress.nextWaypoint = nil
    head.order:get(Components.OrderStatus):setStatus(Status.Active)
end

---The first world of the game: the orders are flown in it.
---@return NavWorldComponent|nil
local function firstWorld()
    for _, world in Registry:view(Components.NavWorld) do return world end
    return nil
end

--- Navigation order system: flies the head order of every ship that has orders. A ship with an order is an
--- agent of the navigation world, and the system writes its control input from the world's axes
--- every update; a ship with no order left leaves the world and drifts. Design: doc/script/nav/navigation.md
---@class NavOrderSystem
---@overload fun(): NavOrderSystem
local NavOrderSystem = Class("NavOrderSystem", function(self)
    local names = {}
    for _, name in pairs(END_EVENTS) do names[#names + 1] = name end
    for _, name in pairs(TRANSITION_EVENTS) do names[#names + 1] = name end
    for _, name in pairs(FLIGHT_EVENTS) do names[#names + 1] = name end
    Event.AddEvents(names)
    for _, name in ipairs(names) do
        EventBus:register(Event[name], name, FrameStage.PostSim)
    end
    self.zones = NavZoneSystem()
    self.rates = NavRateSystem()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end)

---@param data EventData
function NavOrderSystem:onPreSim(data)
    self:update(data:deltaTime())
end

---Ends the orders that are cancelled or whose ship is lost, and returns those that are the head of
---their ship's queue (resuming one that was suspended). An order behind the head waits; one that
---had begun is suspended.
---@param world NavWorldComponent
---@return OrderHead[]
local function collectHeads(world)
    local heads = {}
    for order, navOrder in Registry:view(Components.NavOrder) do
        if not order:get(Components.OrderStatus):hasEnded() then
            local ship = checkOrder(world, order, navOrder)
            -- A ship the player pilots or an AI has claimed flies no order: it leaves the world.
            if ship and not flownByOrders(ship) then detachAgent(world, ship) end
            if ship and flownByOrders(ship) and ship:get(Components.OrderQueue):head() == order.id then
                resumeOrder(order)
                heads[#heads + 1] = { order = order, ship = ship, handler = OrderHandlers[navOrder:getKind()],
                    facing = order:get(Components.FacingGoal) }
            elseif ship and order:get(Components.OrderFlight).begun then
                suspendOrder(order, ship)
            end
        end
    end
    return heads
end

---Steps the world and copies what it computed.
---@param world NavWorldComponent
---@param dt number Seconds
local function stepWorld(world, dt)
    local nav = world:getNav()
    nav:step(dt)
    nav:copyArrived(world:getArrived(), nav:slotCount())
    nav:copyAxes(world:getAxes(), nav:slotCount() * NavWorld.AxisCount())
end

---@param dt number Seconds
function NavOrderSystem:update(dt)
    local world = firstWorld()
    if not world then return end
    dropLostAgents(world)

    local heads = collectHeads(world)
    local zoneSignature = CostZones.signature()
    for _, head in ipairs(heads) do
        head.zoneSignature = zoneSignature
        beginOrder(world, head)
        NavAgentHelper.setFrameFromWell(world:getNav(), head.slot, head.ship)
        if head.facing then aimFacing(world, head) end
        if head.handler and not head.order:get(Components.OrderStatus):hasEnded() then aimGoal(world, head, dt) end
    end
    local changed = self.zones:update()
    self.rates:update(world, dt, heads, changed)
    if not world:hasAgents() then return end
    stepWorld(world, dt)

    for _, head in ipairs(heads) do
        if not head.order:get(Components.OrderStatus):hasEnded() then
            if not head.handler then
                finishHold(world, head)
            elseif head.order:get(Components.OrderFlight).tracking then
                finishTrack(world, head)
            else
                finishGoal(world, head)
            end
        end
    end
end

---Cancels every order that has not ended; used when the world goes.
---@param world NavWorldComponent
function NavOrderSystem:cancelAll(world)
    for order, navOrder in Registry:view(Components.NavOrder) do
        if not order:get(Components.OrderStatus):hasEnded() then
            endOrder(world, order, Registry:getEntityById(navOrder:getShipId()), Status.Cancelled)
        end
    end
end

return NavOrderSystem
