-- What differs between the kinds of navigation order that fly to a goal: the goal, the key that
-- tells a plan is stale, the arrival rule and, for Orbit and Follow, the behaviour kept at the goal.
-- Returns the handlers by `NavOrderKind`; the order system runs them.
-- Design: doc/script/nav/navigation.md

local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local Components          = require("Modules.Navigation.Components")
local NavConfig           = require("Config.Navigation.NavConfig")
local EntityPosition      = require("Shared.Helpers.EntityPosition")
local PlanFlight          = require("Shared.Helpers.PlanFlight")
local ApproachSpecs       = require("Shared.Helpers.ApproachSpecs")
local OrderGoals          = require("Shared.Helpers.OrderGoals")
local PlannerSnapshot     = require("Shared.Helpers.PlannerSnapshot")

local Kind = Enums.NavOrderKind

---How an order that keeps a behaviour of the world on reaching its goal (Orbit, Follow) switches to it.
---@class OrderTrack
---@field start fun(world: NavWorldComponent, head: OrderHead, spec: Component) Gives the ship's agent the behaviour
---@field joins fun(spec: Component, goal: OrderGoal, shipPos: Position, ship: Entity): boolean Whether the ship is near enough to take the behaviour with no plan
---@field leaves fun(spec: Component, goal: OrderGoal, shipPos: Position, ship: Entity): boolean Whether the ship is too far to keep it, and plans again

---What differs between the kinds of order that fly to a goal.
---@class OrderHandler
---@field spec table The component type that holds what the order flies to
---@field goal fun(spec: Component): OrderGoal|nil The goal now; nil when the target is gone
---@field key fun(spec: Component, goal: OrderGoal, shipPos: Position, ship: Entity): integer A number that changes when the goal's place does, enough for a plan to be made again
---@field arrive fun(spec: Component): boolean|nil Takes the order on from an arrival; true when it is done. Nil for an order with a `track`
---@field track OrderTrack|nil Set for an order that never ends and keeps a behaviour once it has reached its goal

local bodyOf, makeGoal, pointGoal, approachGoal =
    OrderGoals.bodyOf, OrderGoals.make, OrderGoals.point, OrderGoals.approach

-- The plane of an orbit with no named normal comes from the ship's velocity about the centre when the
-- ship moves at more than this share of its top speed, and its direction is more than this sine away
-- from the line to the centre; otherwise from the ship's own up axis.
local PLANE_SPEED_SHARE = 0.05
local PLANE_MIN_SINE = 0.1

---@param ship Entity
---@return gameunit
local function followJoinRange(ship)
    return PlanFlight.driveMinLength(ConstructComponents.FlightProfile.ProfileOf(ship))
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
            if not targetId then
                local frameId = spec:getFrameEntityId()
                if not frameId then return pointGoal(spec:getTargetPos()) end
                local frame = Registry:getEntityById(frameId)
                return frame and OrderGoals.framePoint(frame, spec:getFrameOffset()) or nil
            end
            local target = Registry:getEntityById(targetId)
            if not target then return nil end
            return approachGoal(target, ApproachSpecs.For(target, spec:getApproach()))
        end,
        key = function(spec, goal, shipPos)
            -- A body the plan rides on moves with it. Any other entity (a ship) that has moved farther than
            -- a share of its distance has left the place the plan ends at.
            local entity = goal.frameEntity
            if not entity or PlannerSnapshot.isBody(entity) then return spec:getRevision() end
            local range = math.max(NavConfig.goalDriftShare * shipPos:distance(goal.targetPos), goal.arrivalRadius)
            return spec:goalKey(goal.targetPos, range)
        end,
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
        key = function(spec, goal, _, ship) return spec:slotKey(goal.targetPos, followJoinRange(ship)) end,
        track = {
            start = startFollow,
            joins = function(_, goal, shipPos, ship) return shipPos:distance(goal.targetPos) <= followJoinRange(ship) end,
            leaves = function(_, goal, shipPos, ship)
                return shipPos:distance(goal.targetPos) > NavConfig.followLeaveFactor * followJoinRange(ship)
            end,
        },
    },
}

return HANDLERS
