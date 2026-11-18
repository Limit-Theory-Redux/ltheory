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
local ApproachSpecs       = require("Shared.Helpers.ApproachSpecs")
local OrderGoals          = require("Shared.Helpers.OrderGoals")
local ProgressWatch       = require("Shared.Helpers.ProgressWatch")
local NavZoneSystem       = require("Modules.Navigation.Systems.NavZoneSystem")

local Phase   = PlanFlight.Phase
local Status  = Enums.NavOrderStatus
local Failure = Enums.NavOrderFailure
local Kind    = Enums.NavOrderKind

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

---How an order that keeps a behaviour of the world on reaching its goal (Orbit, Follow) switches to it.
---@class OrderTrack
---@field start fun(world: NavWorldComponent, head: OrderHead, spec: Component) Gives the ship's agent the behaviour
---@field joins fun(spec: Component, goal: OrderGoal, shipPos: Position): boolean Whether the ship is near enough to take the behaviour with no plan
---@field leaves fun(spec: Component, goal: OrderGoal, shipPos: Position): boolean Whether the ship is too far to keep it, and plans again

---What differs between the kinds of order that fly to a goal.
---@class OrderHandler
---@field spec table The component type that holds what the order flies to
---@field goal fun(spec: Component): OrderGoal|nil The goal now; nil when the target is gone
---@field key fun(spec: Component, goal: OrderGoal): integer A number that changes when the goal's place does
---@field arrive fun(spec: Component): boolean|nil Takes the order on from an arrival; true when it is done. Nil for an order with a `track`
---@field track OrderTrack|nil Set for an order that never ends and keeps a behaviour once it has reached its goal

local bodyOf, makeGoal, pointGoal, approachGoal =
    OrderGoals.bodyOf, OrderGoals.make, OrderGoals.point, OrderGoals.approach

-- A follower within this many of the shortest leg the travel drive is used on keeps its slot with no plan,
-- and goes back to planning when it is farther than the leave factor times that.
local FOLLOW_LEAVE_FACTOR = 2

-- The plane of an orbit with no named normal comes from the ship's velocity about the centre when the
-- ship moves at more than this share of its top speed, and its direction is more than this sine away
-- from the line to the centre; otherwise from the ship's own up axis.
local PLANE_SPEED_SHARE = 0.05
local PLANE_MIN_SINE = 0.1

---@return gameunit
local function followJoinRange()
    return PlanFlight.driveMinLength(0)
end

---Where a follower's slot is now: its offset from the leader's centre, in the leader's axes.
---@param leader Entity
---@param offset { x: number, y: number, z: number } Game units: right, up, forward
---@return Position
local function slotPoint(leader, offset)
    local body = bodyOf(leader)
    local centre = EntityPosition.Get(leader)
    if not body then return centre end
    local rotation = body:getRot()
    local right, up, forward = rotation:getRight(), rotation:getUp(), rotation:getForward()
    return Position(
        centre.x + right.x * offset.x + up.x * offset.y + forward.x * offset.z,
        centre.y + right.y * offset.x + up.y * offset.y + forward.y * offset.z,
        centre.z + right.z * offset.x + up.z * offset.y + forward.z * offset.z)
end

local function dot(a, b) return a.x * b.x + a.y * b.y + a.z * b.z end
local function length(a) return math.sqrt(dot(a, a)) end
local function cross(a, b)
    return { x = a.y * b.z - a.z * b.y, y = a.z * b.x - a.x * b.z, z = a.x * b.y - a.y * b.x }
end
local function unit(a)
    local len = length(a)
    return { x = a.x / len, y = a.y / len, z = a.z / len }
end

---The plane a ship circles `target` in, as the unit normal it goes anticlockwise about: the plane of its
---position and velocity relative to the target when it moves enough across the line to it; else the
---plane that holds its up axis (or, when that lies along the line, its right axis) and the line.
---@param ship Entity
---@param target Entity
---@return { x: number, y: number, z: number }
local function orbitPlane(ship, target)
    local rb, targetBody = bodyOf(ship), bodyOf(target)
    local at, centre = rb:getPos(), targetBody:getPos()
    local ship_v, target_v = rb:getVelocity(), targetBody:getVelocity()
    local line = { x = at.x - centre.x, y = at.y - centre.y, z = at.z - centre.z }
    local velocity = { x = ship_v.x - target_v.x, y = ship_v.y - target_v.y, z = ship_v.z - target_v.z }
    local lineLength, speed = length(line), length(velocity)
    local normal = cross(line, velocity)
    local top = ConstructComponents.FlightProfile.ProfileOf(ship).maxSpeed
    if speed > PLANE_SPEED_SHARE * top and length(normal) > PLANE_MIN_SINE * lineLength * speed then
        return unit(normal)
    end
    local rotation = rb:getRot()
    for _, axis in ipairs({ rotation:getUp(), rotation:getRight() }) do
        -- The axis with its part along the line taken away.
        local along = lineLength > 0 and dot(axis, line) / (lineLength * lineLength) or 0
        local across = { x = axis.x - line.x * along, y = axis.y - line.y * along, z = axis.z - line.z * along }
        if length(across) > PLANE_MIN_SINE then return unit(across) end
    end
    return unit(rotation:getUp())
end

---Gives the agent the Orbit behaviour about the order's target, in the order's plane: the named one, or
---the ship's own at this moment, which the order keeps from then on.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec OrbitOrderComponent
local function startOrbit(world, head, spec)
    local target = Registry:getEntityById(spec:getTargetEntityId())
    local normal = spec:getChosenNormal()
    if not normal then
        normal = orbitPlane(head.ship, target)
        spec:setChosenNormal(normal)
    end
    world:getNav():setOrbit(head.slot, bodyOf(target), head.goal.safeRange,
        Vec3d(normal.x, normal.y, normal.z), head.goal.arrivalRadius)
end

---Gives the agent the Follow behaviour on the order's leader, at its slot.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec FollowOrderComponent
local function startFollow(world, head, spec)
    local leader = Registry:getEntityById(spec:getLeaderEntityId())
    local offset = spec:getOffset()
    world:getNav():setFollow(head.slot, bodyOf(leader), offset.x, offset.y, offset.z, head.goal.arrivalRadius)
end

---@type table<NavOrderKindId, OrderHandler>
local HANDLERS = {
    [Kind.MoveTo] = {
        spec = Components.MoveToOrder,
        goal = function(spec)
            local targetId = spec:getTargetEntityId()
            if not targetId then return pointGoal(spec:getTargetPos()) end
            local target = Registry:getEntityById(targetId)
            if not target then return nil end
            return approachGoal(target, ApproachSpecs.For(target, spec:getApproach()))
        end,
        key = function(spec) return spec:getRevision() end,
        arrive = function() return true end,
    },
    [Kind.Patrol] = {
        spec = Components.PatrolOrder,
        goal = function(spec) return pointGoal(spec:getPoints()[spec:getIndex()]) end,
        key = function(spec) return spec:getIndex() end,
        arrive = function(spec)
            local index = spec:getIndex()
            if index < #spec:getPoints() then
                spec:setIndex(index + 1)
                return false
            end
            if spec:isLoop() then
                spec:setIndex(1)
                return false
            end
            return true
        end,
    },
    [Kind.Orbit] = {
        spec = Components.OrbitOrder,
        goal = function(spec)
            local target = Registry:getEntityById(spec:getTargetEntityId())
            if not target or not bodyOf(target) then return nil end
            local radius, radii = PlanFlight.targetRadius(target), spec:getAltitude()
            return makeGoal(EntityPosition.Get(target), PlanFlight.standOff(radius, nil, radii),
                PlanFlight.arrivalRadius(radius, nil, radii), target)
        end,
        key = function() return 1 end,
        track = {
            start = startOrbit,
            joins = function() return false end,
            leaves = function() return false end,
        },
    },
    [Kind.Follow] = {
        spec = Components.FollowOrder,
        goal = function(spec)
            local leader = Registry:getEntityById(spec:getLeaderEntityId())
            if not leader or not bodyOf(leader) then return nil end
            return makeGoal(slotPoint(leader, spec:getOffset()), 0, NavConfig.pointArrival(), leader)
        end,
        key = function(spec, goal) return spec:slotKey(goal.targetPos, followJoinRange()) end,
        track = {
            start = startFollow,
            joins = function(_, goal, shipPos) return shipPos:distance(goal.targetPos) <= followJoinRange() end,
            leaves = function(_, goal, shipPos)
                return shipPos:distance(goal.targetPos) > FOLLOW_LEAVE_FACTOR * followJoinRange()
            end,
        },
    },
}

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
    world:unassign(agent:getSlot())
    ship:remove(Components.NavAgent)
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
    local flight = order:get(Components.OrderFlight)
    PlanFlight.drop(flight.plan)
    flight.plan = nil

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
    PlanFlight.drop(flight.plan)
    flight.plan = nil
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

---Gives the order's plan: a new one when there is none or what it was made under has changed. A plan
---that was being flown and is made again is announced, unless the order went on to its next goal.
---@param head OrderHead
---@param spec Component
local function ensurePlan(head, spec)
    local flight = head.order:get(Components.OrderFlight)
    local key = head.handler.key(spec, head.goal)
    local profileRevision = costProfileRevision(head.ship)
    local advanced = flight.advanced
    flight.advanced = false
    if flight.plan and flight.plannedGoalKey == key and flight.plannedProfileRevision == profileRevision then
        return
    end
    local replanned = flight.plan ~= nil and not advanced
    PlanFlight.drop(flight.plan)
    local goal = head.goal
    -- The travel drive thrusts along the nose, which a facing holds elsewhere: the path has no drive leg.
    flight.plan = PlanFlight.begin(head.ship, bodyOf(head.ship), goal.targetPos, goal.safeRange,
        goal.arrivalRadius, goal.frameEntity, head.facing ~= nil)
    flight.plannedGoalKey = key
    flight.plannedProfileRevision = profileRevision
    -- The stall check measures a new plan from the start.
    flight.watchedLeg = nil
    if replanned then EventBus:dispatch(Event[FLIGHT_EVENTS.replanned], head.order.id) end
end

---Moves an order from flying its plan to keeping the world's behaviour: the plan is dropped, and the
---agent takes the behaviour, which the world flies from the next step on. Announces the arrival.
---@param world NavWorldComponent
---@param head OrderHead
---@param spec Component
local function beginTracking(world, head, spec)
    local flight = head.order:get(Components.OrderFlight)
    PlanFlight.drop(flight.plan)
    flight.plan = nil
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
    if flight.tracking and track.leaves(spec, goal, shipPos) then flight.tracking = false end
    if not flight.tracking and track.joins(spec, goal, shipPos) then beginTracking(world, head, spec) end
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
    ensurePlan(head, spec)
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
        not head.facing and PlanFlight.driveWillRun(drive, dist, goal.safeRange))
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
    local leg = atCorners and plan.legs or -1
    if flight.watchedLeg ~= leg then
        flight.watchedLeg = leg
        ProgressWatch.reset(flight.watch, flight.elapsed)
    end
    local dist = atCorners and shipPos:distance(plan.follower.target) or goalDist
    if ProgressWatch.update(flight.watch, dist, flight.elapsed) then
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
                heads[#heads + 1] = { order = order, ship = ship, handler = HANDLERS[navOrder:getKind()],
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
local function stepWorld(world)
    local nav = world:getNav()
    nav:step()
    nav:copyArrived(world:getArrived(), nav:slotCount())
    nav:copyAxes(world:getAxes(), nav:slotCount() * NavWorld.AxisCount())
end

---@param dt number Seconds
function NavOrderSystem:update(dt)
    local world = firstWorld()
    if not world then return end
    dropLostAgents(world)

    local heads = collectHeads(world)
    for _, head in ipairs(heads) do
        beginOrder(world, head)
        NavAgentHelper.setFrameFromWell(world:getNav(), head.slot, head.ship)
        if head.facing then aimFacing(world, head) end
        if head.handler and not head.order:get(Components.OrderStatus):hasEnded() then aimGoal(world, head, dt) end
    end
    self.zones:update()
    if not world:hasAgents() then return end
    stepWorld(world)

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
