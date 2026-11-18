-- Navigation orders: what a ship is told to do, as an order entity linked to the ship. The issuer
-- gets the order's entity, reads its status and progress, and releases it when it has ended.
-- Design: doc/script/nav/navigation.md

local Registry            = require("Core.ECS.Registry")
local Entity              = require("Core.ECS.Entity")
local PhysicsComponents   = require("Modules.Physics.Components")
local Components          = require("Modules.Navigation.Components")
local Entities            = require("Modules.Navigation.Entities")
local Systems             = require("Modules.Navigation.Systems")

local Kind = Enums.NavOrderKind

-- The collision mask the navigation world senses obstacles with: every collision group.
local SENSING_MASK = 0xFFFFFFFF

---@class Nav
local Nav = { Components = Components, Orders = {}, Priority = Enums.NavOrderPriority }

---@type Entity|nil The entity that holds the navigation world
local worldEntity = nil
---@type NavOrderSystem|nil Created with the first world, and kept for the life of the game
local system = nil

---Creates the navigation world over `physics`, which steers the ships that have orders. The ship
---bodies must stay in `physics` while their orders run. There is one world at a time.
---@param physics Physics
function Nav.attach(physics)
    assert(not worldEntity, "Nav.attach: there is a navigation world already (Nav.detach it first)")
    local component = Components.NavWorld(physics)
    component:getNav():setSensing(SENSING_MASK)
    worldEntity = Entity.Create("NavWorldEntity", component)
    system = system or Systems.NavOrder()
end

---Cancels the orders that have not ended and destroys the navigation world. Call it before the
---ships' bodies leave physics.
function Nav.detach()
    assert(worldEntity, "Nav.detach: there is no navigation world")
    system:cancelAll(worldEntity:get(Components.NavWorld))
    Registry:destroyEntity(worldEntity)
    worldEntity = nil
end

---What the nose is to face while an order runs, apart from where the ship flies. While an order has
---one, the ship flies without the travel drive.
---@class NavFacing
---@field target NavTarget An entity, which the nose follows, or a point

---Flies to a target: an entity, which the ship stops at the approach of (its kind's own when none
---is named), or a point.
---@param spec { target: NavTarget, approach: ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil, facing: NavFacing|nil, priority: NavOrderPriorityId|nil } `approach`: for an entity target only; `priority`: an `Nav.Priority` level, `Routine` when nil. Every order takes `priority`
---@return table
function Nav.Orders.MoveTo(spec)
    return { kind = Kind.MoveTo, target = spec.target, approach = spec.approach, facing = spec.facing,
        priority = spec.priority }
end

---Flies through points in turn.
---@param spec { points: Position[], loop: boolean|nil, facing: NavFacing|nil, priority: NavOrderPriorityId|nil } `loop`: start over after the last point
---@return table
function Nav.Orders.Patrol(spec)
    return { kind = Kind.Patrol, points = spec.points, loop = spec.loop, facing = spec.facing,
        priority = spec.priority }
end

---Circles an entity, anticlockwise about a normal, and never ends: cancel it. The ship flies the system
---plan to the circle first, then holds it as the entity moves.
---@param spec { target: Entity, altitude: number|nil, normal: Vec3d|nil, facing: NavFacing|nil, priority: NavOrderPriorityId|nil } `altitude`: target radii from its centre, more than 1 (`NavConfig.standOffRadii` when nil); `normal`: the circle's plane (the plane the ship is in when it reaches the circle when nil)
---@return table
function Nav.Orders.Orbit(spec)
    return { kind = Kind.Orbit, target = spec.target, altitude = spec.altitude, normal = spec.normal,
        facing = spec.facing, priority = spec.priority }
end

---Keeps a slot at `offset` from a leader, in the leader's axes, matching its velocity, and never ends:
---cancel it. The ship flies the system plan to the slot while it is far from it.
---@param spec { target: Entity, offset: Vec3d, facing: NavFacing|nil, priority: NavOrderPriorityId|nil } `offset`: metres, right, up, forward
---@return table
function Nav.Orders.Follow(spec)
    return { kind = Kind.Follow, target = spec.target, offset = spec.offset, facing = spec.facing,
        priority = spec.priority }
end

---Keeps the ship where it is, and never ends: cancel it.
---@param spec { facing: NavFacing|nil, priority: NavOrderPriorityId|nil }|nil
---@return table
function Nav.Orders.Hold(spec)
    return { kind = Kind.Hold, facing = spec and spec.facing, priority = spec and spec.priority }
end

---Gives a ship an order: behind the orders of its priority or a higher one, and before the lower
---ones. An order of a higher priority than the one the ship is flying displaces it, which resumes
---when the order that displaced it has ended.
---@param ship Entity
---@param spec table From `Nav.Orders`
---@return Entity order
function Nav.order(ship, spec)
    assert(worldEntity, "Nav.order: there is no navigation world (Nav.attach it)")
    assert(ship:get(PhysicsComponents.RigidBody), "Nav.order: the ship has no rigid body")
    local order = Entities.NavOrder(ship.id, spec)
    local queue = ship:get(Components.OrderQueue)
    if not queue then
        queue = Components.OrderQueue()
        ship:add(queue)
    end
    queue:push(order.id, order:get(Components.NavOrder):getPriority())
    return order
end

---Runs one update of the order system, for an app that steps the simulation itself; a game
---that runs frames gets the update at PreSim, so it must not call this too.
---@param dt number Seconds
function Nav.update(dt)
    system:update(dt)
end

---Asks for the order to end as cancelled; the system ends it at its next update.
---@param order Entity
function Nav.cancel(order)
    order:get(Components.OrderStatus):requestCancel()
end

---Changes the target of a MoveTo order; the ship's path is planned again.
---@param order Entity
---@param target NavTarget
function Nav.retarget(order, target)
    local moveTo = order:get(Components.MoveToOrder)
    assert(moveTo, "Nav.retarget: not a MoveTo order")
    moveTo:setTarget(target)
end

---@param order Entity
---@return NavOrderStatusId
function Nav.status(order)
    return order:get(Components.OrderStatus):getStatus()
end

---@param order Entity
---@return NavOrderFailureId|nil Why the order failed; nil while it has not
function Nav.failure(order)
    return order:get(Components.OrderStatus):getFailure()
end

---@param order Entity
---@return OrderProgressComponent
function Nav.progress(order)
    return order:get(Components.OrderProgress)
end

---Destroys an order that has ended. The issuer calls it when it has read what it wants of the order.
---@param order Entity
function Nav.release(order)
    local status = order:get(Components.OrderStatus)
    assert(status and status:hasEnded(), "Nav.release: the order has not ended (cancel it first)")
    Registry:destroyEntity(order)
end

return Nav
