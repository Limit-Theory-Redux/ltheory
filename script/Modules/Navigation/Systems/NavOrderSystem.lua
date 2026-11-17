local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local Components          = require("Modules.Navigation.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local NavConfig           = require("Config.Navigation.NavConfig")
local EntityPosition      = require("Shared.Helpers.EntityPosition")
local NavAgentHelper      = require("Shared.Helpers.NavAgentHelper")
local PlanFlight          = require("Shared.Helpers.PlanFlight")

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

---Where a goal order flies to, and how it counts as arrived.
---@class OrderGoal
---@field targetPos Position The goal now
---@field frameBody RigidBody|nil The body the goal rides on
---@field frameEntity Entity|nil The entity that body belongs to
---@field safeRange gameunit The stand-off distance from the target's centre; 0 for a point
---@field arrivalRadius gameunit

---@class OrderHead A ship's head order, with what the frame needs of it
---@field order Entity
---@field ship Entity
---@field slot integer The ship's agent
---@field handler OrderHandler|nil Nil for a Hold
---@field goal OrderGoal|nil

---What differs between the kinds of order that fly to a goal.
---@class OrderHandler
---@field spec table The component type that holds what the order flies to
---@field goal fun(spec: Component): OrderGoal|nil The goal now; nil when the target is gone
---@field key fun(spec: Component): integer A number that changes when the goal's place does
---@field arrive fun(spec: Component): boolean Takes the order on from an arrival; true when it is done

---@param entity Entity
---@return RigidBody|nil
local function bodyOf(entity)
    local component = entity:get(PhysicsComponents.RigidBody)
    return component and component:getRigidBody() or nil
end

---@param targetPos Position
---@param safeRange gameunit
---@param arrivalRadius gameunit
---@param frame Entity|nil
---@return OrderGoal
local function makeGoal(targetPos, safeRange, arrivalRadius, frame)
    return {
        targetPos = targetPos,
        frameEntity = frame,
        frameBody = frame and bodyOf(frame) or nil,
        safeRange = safeRange,
        arrivalRadius = arrivalRadius,
    }
end

---@param point Position
---@return OrderGoal
local function pointGoal(point)
    return makeGoal(point, 0, NavConfig.pointArrival(), nil)
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
            local radius = PlanFlight.targetRadius(target)
            local gap = spec:getGap()
            return makeGoal(EntityPosition.Get(target), PlanFlight.standOff(radius, gap),
                PlanFlight.arrivalRadius(radius, gap), target)
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
}

---@param ship Entity
---@return integer The revision of the ship's cost profile, 0 for a ship without one
local function costProfileRevision(ship)
    local profile = ship:get(Components.CostProfile)
    return profile and profile:getRevision() or 0
end

---Takes a ship's agent out of the world and gives the controls back: no axes left over, the
---travel drive stopped and its cap lifted.
---@param world NavWorldComponent
---@param ship Entity
local function detachAgent(world, ship)
    local agent = ship:get(Components.NavAgent)
    if not agent then return end
    world:getNav():removeAgent(agent:getSlot())
    world:unassign(agent:getSlot())
    ship:remove(Components.NavAgent)
    local input = ship:get(ConstructComponents.ControlInput)
    if input then
        input:clearAxes()
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
        input:setDriveSpeedCap(nil)
    end
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
        local input = ship:get(ConstructComponents.ControlInput)
        if input then input:setDriveRequest(Enums.TravelDriveRequest.Disengage) end
        if queue:count() == 0 then detachAgent(world, ship) end
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

---Starts a head order that has not begun: the ship gets its agent, and a Hold its goal.
---@param world NavWorldComponent
---@param head OrderHead
local function beginOrder(world, head)
    local flight = head.order:get(Components.OrderFlight)
    attachAgent(world, head.ship, bodyOf(head.ship))
    head.slot = head.ship:get(Components.NavAgent):getSlot()
    if flight.begun then return end
    flight.begun = true
    if not head.handler then
        world:getNav():setHold(head.slot, NavConfig.pointArrival())
        head.order:get(Components.OrderStatus):setStatus(Status.Active)
    end
end

---Gives the order's plan: a new one when there is none or what it was made under has changed.
---@param head OrderHead
---@param spec Component
local function ensurePlan(head, spec)
    local flight = head.order:get(Components.OrderFlight)
    local key = head.handler.key(spec)
    local profileRevision = costProfileRevision(head.ship)
    if flight.plan and flight.plannedGoalKey == key and flight.plannedProfileRevision == profileRevision then
        return
    end
    PlanFlight.drop(flight.plan)
    local goal = head.goal
    flight.plan = PlanFlight.begin(head.ship, bodyOf(head.ship), goal.targetPos, goal.safeRange,
        goal.arrivalRadius, goal.frameEntity)
    flight.plannedGoalKey = key
    flight.plannedProfileRevision = profileRevision
end

---Before the step of a goal order: its plan, and its goal on the world unless the plan's corners are flown.
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
    ensurePlan(head, spec)
    local flight = head.order:get(Components.OrderFlight)
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
    if drive and input and plan.phase == Phase.Plan then
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
    -- The drive runs on the final approach and a flight with no usable plan, never while the plan is out.
    if drive and input and plan.phase ~= Phase.Planning then
        PlanFlight.updateDrive(drive, input, profile, zoneMax, dist, goal.safeRange)
    end
    return PlanFlight.travelTime(math.max(0, dist - goal.safeRange), goal.arrivalRadius, profile, zoneMax,
        PlanFlight.driveWillRun(drive, dist, goal.safeRange))
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
        if head.handler.arrive(order:get(head.handler.spec)) then
            endOrder(world, order, ship, Status.Done)
        end
        return
    end
    local eta
    if atCorners then
        eta = flyCorners(head, plan, flight, arrived, zoneMax)
    else
        eta = flyApproach(head, plan, dist, zoneMax)
    end
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
    Event.AddEvents({ "NavOrderDone", "NavOrderFailed", "NavOrderCancelled" })
    for _, name in pairs(END_EVENTS) do
        EventBus:register(Event[name], name, FrameStage.PostSim)
    end
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end)

---@param data EventData
function NavOrderSystem:onPreSim(data)
    self:update(data:deltaTime())
end

---Ends the orders that are cancelled or whose ship is lost, and returns those that are the head of
---their ship's queue. An order behind the head waits.
---@param world NavWorldComponent
---@return OrderHead[]
local function collectHeads(world)
    local heads = {}
    for order, navOrder in Registry:view(Components.NavOrder) do
        if not order:get(Components.OrderStatus):hasEnded() then
            local ship = checkOrder(world, order, navOrder)
            if ship and ship:get(Components.OrderQueue):head() == order.id then
                heads[#heads + 1] = { order = order, ship = ship, handler = HANDLERS[navOrder:getKind()] }
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
        if head.handler then aimGoal(world, head, dt) end
    end
    if not world:hasAgents() then return end
    stepWorld(world)

    for _, head in ipairs(heads) do
        if not head.order:get(Components.OrderStatus):hasEnded() then
            if head.handler then finishGoal(world, head) else finishHold(world, head) end
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
