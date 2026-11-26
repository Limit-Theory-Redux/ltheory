-- The behaviours a sandbox ship can be given besides flying to a target (Seek, Hold, Pursue,
-- Follow, Orbit): issuing each one to the `NavWorld`, the point the ship is to be at, and the
-- verdict on whether it has done what the behaviour asks. A ship can also be flown by orders
-- (`Nav.order`); it is then judged by what its last order asks, with the same rules. The geometry
-- here is the sandbox's own, written from the behaviour's definition, so it checks the engine's
-- maths instead of repeating it.
-- Design: doc/script/nav/navigation-sandbox.md

local Metres               = require("States.App.Tests.NavigationSandbox.Support.Metres")
local ConstructComponents  = require("Modules.Constructs.Components")
local Nav                  = require("Modules.Navigation")
local NavigationComponents = require("Modules.Navigation.Components")
local SystemPlanner        = require("Shared.Helpers.SystemPlanner")
local PlanFollower         = require("Shared.Helpers.PlanFollower")
local ProgressWatch        = require("Shared.Helpers.ProgressWatch")

-- A ship that tracks a moving point has done so when it has stayed on station this long.
local STATION_HOLD_S      = 5
-- A Seek point is passed at least this fraction of top speed: Arrive would be braking there.
local PASS_SPEED_FRACTION = 0.5
-- A capped ship is within its cap, and a Seek point with a pass speed is passed within it, up to
-- this fraction over. The cap is held by the controller's speed limit; a pass speed is the end of
-- a braking curve that the velocity loop follows with a lag.
local CAP_TOLERANCE         = 0.05
local PASS_SPEED_TOLERANCE  = 0.35
-- A turn is measured at speeds above this fraction of top speed, and over a rotation of the
-- velocity above this many radians.
local MIN_TURN_SPEED_FRACTION = 0.1
local MIN_TURN_ANGLE          = 1e-4
-- An orbiting ship has orbited after it has turned this far about the centre.
local FULL_TURN           = 2 * math.pi
-- An orbiting ship is on its circle within this fraction of the radius: the wanted velocity is
-- the circle's tangent, and a ship at kilometres per second runs a little wide of it.
local ORBIT_TOLERANCE     = 0.1
-- A direction is taken from a vector at least this long, relative to the radius it is compared with.
local MIN_RELATIVE_LENGTH = 1e-6

---One order a ship is given, in the order of `SandboxBehaviourSpec.orders`. The ship is judged by the last one:
---MoveTo and Patrol by the order being done, Hold by the ship holding its place, Orbit and Follow as the
---behaviours of that name are, with the fields they have here.
---@class SandboxOrderSpec
---@field kind "MoveTo"|"Patrol"|"Hold"|"Orbit"|"Follow"
---@field obstacle integer|nil MoveTo, Orbit: index of the scenario's obstacle that is flown to or circled
---@field point number[]|nil MoveTo: { x, y, z } in metres, when there is no obstacle nor ship
---@field gapM number|nil MoveTo to a ship or obstacle: the gap between the hulls the ship stops at, in metres, instead of the target kind's own approach
---@field points number[][]|nil Patrol: points { x, y, z } in metres
---@field loop boolean|nil Patrol
---@field altitude number|nil Orbit: target radii from the obstacle's centre
---@field radiusM number|nil Orbit: the radius of the circle the ship is judged on, in metres; written from the scenario, not from the order's maths
---@field turnRad number|nil Orbit: the ship has orbited after it has turned this far, in radians; a full turn when nil
---@field normal number[]|nil Orbit: { x, y, z } normal of the order's plane, and the plane the ship is judged on; the plane the order chooses when nil
---@field ship integer|nil MoveTo, Follow: index of the scenario's ship that is flown to or followed
---@field offsetM number[]|nil Follow: the slot { right, up, forward } in the leader's axes, in metres
---@field priority"Routine"|"Urgent"|"Combat"|nil Any order: its `Nav.Priority` level
---@field afterS number|nil Any order: it is given when the scenario has run this many seconds, not at the start
---@field judged boolean|nil The ship is judged by this order, not the last one
---@field suspends boolean|nil The judged order is displaced by a later one: the ship has arrived when the order has been suspended, then done
---@field choose SandboxChoiceSpec|nil MoveTo, instead of `point`: the point is chosen from candidates by their estimates when the order is due

---Candidates for a MoveTo point, scored with `Nav.estimateMany` as time plus `riskWeightSPerM` times the risk
---(the cost zones' cost times the length crossed, in metres); the lowest score among the reachable ones wins.
---@class SandboxChoiceSpec
---@field points number[][] Candidate points { x, y, z } in metres
---@field riskWeightSPerM number Seconds a metre of weighted risk is worth

---@class SandboxBehaviourSpec
---@field kind "Seek"|"Hold"|"Pursue"|"Follow"|"Orbit"|"Plan"|"Orders"
---@field orders SandboxOrderSpec[]|nil Orders: given to the ship in this order. The ship is not an agent of the sandbox's `NavWorld`; `Nav` flies it
---@field through number[][]|nil Seek: points { x, y, z } in metres, passed at speed in this order; the ship then stops at the ship's `target`
---@field passRadiusM number|nil Seek, Plan: a point is passed within this distance, in metres; longer than the distance covered in one frame
---@field speedCapShare number|nil Seek: the ship's speed cap, as a fraction of top speed; no cap when nil. Flying faster than it fails the run
---@field passSpeedShare number|nil Seek: each point is passed at this fraction of top speed and the ship brakes for it; top speed, without braking, when nil
---@field measureTurn boolean|nil Seek: log the tightest turn the ship flew, with the speed and the thrust share it implies
---@field holdAtSpeed number|nil Hold: the ship holds when it flies at this fraction of top speed on its way to `target`
---@field obstacle integer|nil Pursue, Orbit: index of the scenario's obstacle that is chased or circled
---@field ship integer|nil Follow: index of the scenario's ship that is followed
---@field standOffM number|nil Pursue: gap kept between the hulls, in metres
---@field offsetM number[]|nil Follow: the slot { right, up, forward } in the leader's axes, in metres
---@field radiusM number|nil Orbit: radius of the circle, in metres
---@field normal number[]|nil Orbit: { x, y, z } normal of the circle's plane; the ship goes anticlockwise about it

---@class SandboxBehaviour
---@field kind string The rule the ship is judged by; for a ship flown by orders, that of its last order: "OrderDone", "OrderHold", "Orbit" or "Follow"
---@field spec SandboxBehaviourSpec For a ship flown by orders, the last order's fields, with `kind` as above and `orders` the whole list
---@field orders Entity[]|nil Ships flown by orders: the order entities, in the order they were given
---@field holdPoint table|nil OrderHold: { x, y, z } where the ship was when its Hold order began
---@field turnLimit number|nil Orbit: radians turned about the centre after which the ship has orbited
---@field tracked table|nil The sandbox ship or obstacle the behaviour is about
---@field topSpeed number Game units per second
---@field engaged boolean Whether the ship has been on station; deadlocks are judged only before
---@field maxError number Largest distance from the station point since it was engaged, in game units
---@field phase string|nil Seek: "pass" or "arrive". Plan: "waiting" for the plan, then as Seek. Hold: "cruise", "switched" or "returning"
---@field points Position[]|nil Seek: the points to pass
---@field pointIndex integer|nil Seek: the point being flown to
---@field passRadius number|nil Seek, Plan: game units
---@field passSpeed number Seek: game units per second a point is passed at; `math.huge` for top speed
---@field speedCap number|nil Seek: game units per second, as given to the `NavWorld`
---@field peakSpeed number Seek: the largest speed the ship has flown at, in game units per second
---@field turn SandboxTurn|nil Seek with `measureTurn`: the tightest turn so far
---@field startPoint table|nil Plan: { x, y, z } where the ship was when it asked for the plan, the point before the first corner
---@field follower PlanFollower|nil Plan: flies the corners once the plan has come
---@field listener table|nil Plan: what the follower reports to
---@field goal PlanWaypoint|nil Plan: where the ship stops; with a frame, a point that rides on that obstacle
---@field obstacles SandboxObstacle[]|nil Plan: the scenario's obstacles, which waypoint frames index
---@field clearance table<integer, number>|nil Plan, Orders with `expect.zone`: per obstacle index, the smallest distance from the ship's centre to the obstacle's safe sphere since the start, in game units
---@field zoneClearance table<integer, number>|nil Plan, Orders with `expect.zone`: per cost zone index, the smallest distance from the ship's centre to the zone's sphere since the start (negative: inside it), in game units
---@field replanned integer|nil Orders: how many times the judged order announced `NavOrderReplanned`
---@field replannedTunnel integer|nil Orders: the subscription that counts `replanned`
---@field planTime number|nil Plan: simulated seconds when the plan being made or flown was requested; the plan's times are from then
---@field revision integer|nil Plan: the revision of the ship's cost profile the plan was requested under; nil for a ship with none
---@field plansTaken integer|nil Plan: plans received and taken
---@field replans integer|nil Plan: requests made again because the cost profile changed
---@field onSince number|nil Simulated time the ship has been on station since
---@field onStation boolean|nil Pursue, Follow: whether the ship is on station now, whether or not it is yet judged
---@field turnRate number|nil Pursue, Follow: largest turn rate, in radians per second, since the ship was last off station or not yet judged
---@field turned number Orbit: radians turned about the centre since the ship was last off the circle
---@field previous table|nil Orbit: the ship's last offset from the axis, in the plane of the circle
---@field normal table|nil Orbit: unit normal
---@field radius number|nil Orbit: game units
---@field offset table|nil Follow: the slot, game units
---@field standOff number|nil Pursue: game units

---@class SandboxTurn
---@field radius number Game units: the smallest turn radius seen, speed over the rate the velocity turns at
---@field speed number Game units per second at that radius
---@field velocity table|nil The velocity at the last step, { x, y, z }
---@field time number|nil Simulated time of the last step

---@class SandboxJudgingRules
---@field range number Game units: within this of its point, a ship is there
---@field speed number Game units per second: at most this fast (relative to a moving target), it is at rest there

---@class ShipBehaviour
local ShipBehaviour = {}

local function sub(a, b) return { x = a.x - b.x, y = a.y - b.y, z = a.z - b.z } end
local function add(a, b) return { x = a.x + b.x, y = a.y + b.y, z = a.z + b.z } end
local function scale(a, s) return { x = a.x * s, y = a.y * s, z = a.z * s } end
local function dot(a, b) return a.x * b.x + a.y * b.y + a.z * b.z end
local function length(a) return math.sqrt(dot(a, a)) end
local function cross(a, b)
    return { x = a.y * b.z - a.z * b.y, y = a.z * b.x - a.x * b.z, z = a.x * b.y - a.y * b.x }
end

---@param v table { x, y, z }
---@return Position
local function toPosition(v) return Position(v.x, v.y, v.z) end

---The unit vector along `v`, or `fallback` when `v` is shorter than `minLength`.
local function unitOr(v, minLength, fallback)
    local len = length(v)
    return len > minLength and scale(v, 1 / len) or fallback
end

---Some unit vector across `n`, for the direction of an offset that has none.
local function anyAcross(n)
    local across = cross(n, { x = 1, y = 0, z = 0 })
    if length(across) < MIN_RELATIVE_LENGTH then across = cross(n, { x = 0, y = 1, z = 0 }) end
    return scale(across, 1 / length(across))
end

---@param item table A sandbox ship or obstacle
---@return number Game units
local function radiusOf(item)
    return item.radius or item.rb:getBoundingRadius()
end

-- The rule a ship flown by orders is judged by, from the kind of the order it is judged by.
local ORDER_RULES = { MoveTo = "OrderDone", Patrol = "OrderDone", Hold = "OrderHold", Orbit = "Orbit", Follow = "Follow" }

---@param spec SandboxBehaviourSpec
---@return SandboxBehaviour
function ShipBehaviour.create(spec)
    if spec.kind == "Orders" then
        -- The order the ship is judged by: the one marked, else the last.
        local judgedIndex = #spec.orders
        for i, order in ipairs(spec.orders) do
            if order.judged then judgedIndex = i end
        end
        local last = spec.orders[judgedIndex]
        local judged = {}
        for key, value in pairs(last) do judged[key] = value end
        judged.kind = assert(ORDER_RULES[last.kind], "ShipBehaviour: unknown order kind " .. tostring(last.kind))
        judged.orders = spec.orders
        judged.judgedIndex = judgedIndex
        spec = judged
    end
    return {
        kind = spec.kind, spec = spec, engaged = false, maxError = 0, turned = 0, topSpeed = 0,
        passSpeed = math.huge, peakSpeed = 0, turnLimit = spec.turnRad or FULL_TURN,
    }
end

---Whether the ship is flown by orders, not as an agent of the sandbox's `NavWorld`.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.isOrdered(ship)
    return ship.behaviour ~= nil and ship.behaviour.spec.orders ~= nil
end

---Sends a Seek or Plan ship through its current point.
---@param nav NavWorld
---@param ship SandboxShip
local function seekPoint(nav, ship)
    local b = ship.behaviour
    nav:setSeek(ship.agent, ship.target, b.passRadius, b.passSpeed)
    nav:refresh(ship.agent)
end

---Where a ship tracking a moving point is to be now, in the sandbox's own terms.
---@param b SandboxBehaviour
---@param ship SandboxShip
---@return table { x, y, z }
local function stationPoint(b, ship)
    local shipPos = ship.rb:getPos()
    local trackedPos = b.tracked.rb:getPos()
    if b.kind == "Pursue" then
        -- On the line of sight, a hull gap from the target's hull.
        local outward = unitOr(sub(shipPos, trackedPos), 0, { x = 0, y = 0, z = 1 })
        local gap = radiusOf(b.tracked) + ship.rb:getBoundingRadius() + b.standOff
        return add(trackedPos, scale(outward, gap))
    elseif b.kind == "Follow" then
        local rotation = b.tracked.rb:getRot()
        local right, up, forward = rotation:getRight(), rotation:getUp(), rotation:getForward()
        return add(trackedPos, add(add(scale(right, b.offset.x), scale(up, b.offset.y)),
            scale(forward, b.offset.z)))
    end
    -- Orbit: the nearest point of the circle, or of the sphere while the plane is not known yet.
    local offset = sub(shipPos, trackedPos)
    local planar = b.normal and sub(offset, scale(b.normal, dot(offset, b.normal))) or offset
    local direction = unitOr(planar, b.radius * MIN_RELATIVE_LENGTH, anyAcross(b.normal or { x = 0, y = 1, z = 0 }))
    return add(trackedPos, scale(direction, b.radius))
end

local TRACKING = { Pursue = true, Follow = true, Orbit = true }

---The world point of a plan's goal now: its offset from its frame's obstacle, or the point itself.
---@param b SandboxBehaviour
---@return Position
local function goalNow(b)
    local goal = b.goal
    if not goal.frame then return goal.offset end
    local centre, offset = b.obstacles[goal.frame].rb:getPos(), goal.offset
    return Position(centre.x + offset.x, centre.y + offset.y, centre.z + offset.z)
end

---The point a Plan ship is to be at now, which is what its progress is judged against: the
---follower's point while it flies the corners, then the goal.
---@param b SandboxBehaviour
---@return Position
local function planPoint(b)
    if b.phase ~= "pass" then return goalNow(b) end
    return b.follower:point()
end

---Gives a Plan ship the goal as its arrival target: a goal that rides on an obstacle keeps its
---place relative to it, so the ship matches the obstacle's velocity there.
---@param nav NavWorld
---@param ship SandboxShip
---@param radius number Game units
local function arriveAtGoal(nav, ship, radius)
    local b = ship.behaviour
    local goal = b.goal
    if goal.frame then
        local offset = goal.offset
        nav:setArriveRelative(ship.agent, b.obstacles[goal.frame].rb, Vec3d(offset.x, offset.y, offset.z), radius)
    else
        nav:setArrive(ship.agent, goal.offset, radius)
    end
    nav:refresh(ship.agent)
end

---The point a ship flown by orders is shown to fly to: where its MoveTo goes, the last point of its
---Patrol, or, for a Hold, where the ship was when the Hold began (the ship itself before).
---@param b SandboxBehaviour
---@param ship SandboxShip
---@return Position
local function orderPoint(b, ship)
    local spec = b.spec
    if b.kind == "OrderHold" then
        if not b.holdPoint and b.judgedOrder and Nav.status(b.judgedOrder) == Enums.NavOrderStatus.Active then
            local at = ship.rb:getPos()
            b.holdPoint = { x = at.x, y = at.y, z = at.z }
        end
        return b.holdPoint and toPosition(b.holdPoint) or ship.rb:getPos()
    end
    if b.chosenPoint then return Metres.position(b.chosenPoint) end
    if spec.choose then return ship.rb:getPos() end
    if spec.obstacle or spec.ship then return b.tracked.rb:getPos() end
    return Metres.position(spec.point or spec.points[#spec.points])
end

---Takes the plane an Orbit order chose, once it has chosen one, as the plane the ship is judged on.
---@param b SandboxBehaviour
local function takeChosenNormal(b)
    if b.normal or not b.judgedOrder then return end
    local chosen = b.judgedOrder:get(NavigationComponents.OrbitOrder):getChosenNormal()
    b.normal = chosen
end

---Sets `ship.target` to the point a tracking or Plan ship is to be at, which moves with its body;
---other ships keep theirs.
---@param ship SandboxShip
function ShipBehaviour.refreshTarget(ship)
    local b = ship.behaviour
    if not b then return end
    if b.orders and b.kind == "Orbit" then takeChosenNormal(b) end
    if TRACKING[b.kind] then
        ship.target = toPosition(stationPoint(b, ship))
    elseif b.kind == "Plan" then
        ship.target = planPoint(b)
    elseif b.orders then
        ship.target = orderPoint(b, ship)
    end
end

---Whether the ship keeps its behaviour after it has done what it asks: a tracking ship goes on
---tracking, the others are released and come to rest.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.keepsBehaviour(ship)
    return TRACKING[ship.behaviour.kind] == true
end

---Whether the ship's lack of progress is not to be judged now: a tracking ship that has been on
---station moves with its point, and a held ship overshoots its point before it comes back. A ship
---that never gets there is caught by the time limit.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.exemptFromDeadlock(ship)
    local b = ship.behaviour
    -- An order ship's end is its order's: a failed order is reported, and one that never ends is caught
    -- by the time limit.
    return b.spec.orders ~= nil or (TRACKING[b.kind] and b.engaged) or (b.kind == "Hold" and b.phase ~= "cruise")
end

---Whether an order of the ship has ended without being done: failed, or cancelled.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.orderFailed(ship)
    local orders = ship.behaviour.orders
    if not orders then return false end
    for _, order in ipairs(orders) do
        local status = Nav.status(order)
        if status == Enums.NavOrderStatus.Failed or status == Enums.NavOrderStatus.Cancelled then return true end
    end
    return false
end

---Releases the ship's orders once the navigation world is detached, which ends them all.
---@param ship SandboxShip
function ShipBehaviour.releaseOrders(ship)
    local b = ship.behaviour
    for _, order in ipairs(b and b.orders or {}) do Nav.release(order) end
    if b and b.replannedTunnel then
        EventBus:unsubscribe(b.replannedTunnel)
        b.replannedTunnel = nil
    end
end

---The revision of the ship's cost profile, nil when it has none.
---@param ship SandboxShip
---@return integer|nil
local function costRevision(ship)
    local component = ship.entity:get(NavigationComponents.CostProfile)
    return component and component:getRevision() or nil
end

---Sends a Plan ship's request from where it is now, and has it fly at the goal until the plan comes.
---@param nav NavWorld
---@param ship SandboxShip
---@param rules SandboxJudgingRules
---@param planner PlanClient
---@param simTime number
local function requestPlan(nav, ship, rules, planner, simTime)
    local b = ship.behaviour
    b.phase, b.planTime, b.revision = "waiting", simTime, costRevision(ship)
    arriveAtGoal(nav, ship, rules.range)
    local at = ship.rb:getPos()
    b.startPoint = { x = at.x, y = at.y, z = at.z }
    planner:request(ship, b.goal.offset, b.goal.frame, b.topSpeed, b.obstacles, simTime, rules.range)
end

---@param order SandboxOrderSpec
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
---@return table An order from `Nav.Orders`
local function navOrder(order, ships, obstacles)
    local kind = order.kind
    local priority = order.priority and assert(Nav.Priority[order.priority], "ShipBehaviour: unknown priority " .. order.priority)
    if kind == "MoveTo" then
        return Nav.Orders.MoveTo {
            target = order.ship and ships[order.ship].entity
                or order.obstacle and obstacles[order.obstacle].entity
                or Metres.position(order.point),
            approach = order.gapM and { kind = Enums.ApproachKind.StandOff, gapM = order.gapM },
            priority = priority,
        }
    elseif kind == "Patrol" then
        local points = {}
        for _, point in ipairs(order.points) do table.insert(points, Metres.position(point)) end
        return Nav.Orders.Patrol { points = points, loop = order.loop, priority = priority }
    elseif kind == "Hold" then
        return Nav.Orders.Hold { priority = priority }
    elseif kind == "Orbit" then
        local n = order.normal
        return Nav.Orders.Orbit {
            target = obstacles[order.obstacle].entity, altitude = order.altitude, normal = n and Vec3d(n[1], n[2], n[3]),
            priority = priority,
        }
    end
    local o = order.offsetM
    return Nav.Orders.Follow { target = ships[order.ship].entity, offset = Vec3d(o[1], o[2], o[3]), priority = priority }
end

---Gives the ship the order at `index` of its spec (or `spec` in its place), and keeps the one it is judged by.
---@param ship SandboxShip
---@param index integer
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
---@param spec SandboxOrderSpec|nil
local function giveOrder(ship, index, ships, obstacles, spec)
    local b = ship.behaviour
    local order = Nav.order(ship.entity, navOrder(spec or b.spec.orders[index], ships, obstacles))
    table.insert(b.orders, order)
    if index == b.spec.judgedIndex then
        b.judgedOrder, b.replanned = order, 0
        b.replannedTunnel = EventBus:subscribe(Event.NavOrderReplanned, nil, function(_, _, id)
            if id == order.id then b.replanned = b.replanned + 1 end
        end)
    end
end

---Asks for the estimates of the candidates of the order at `index`, which is given once they have come.
---@param ship SandboxShip
---@param index integer
local function startChoice(ship, index)
    local b = ship.behaviour
    local candidates = {}
    for i, point in ipairs(b.spec.orders[index].choose.points) do candidates[i] = Metres.position(point) end
    b.choosing = { index = index, request = Nav.estimateMany(ship.entity, candidates) }
end

---The score of a candidate: its time plus its risk in metres at `weight` seconds a metre.
---@param estimate NavEstimateResult Reachable
---@param weight number
---@return number
local function scoreOf(estimate, weight)
    return estimate.time + weight * Metres.toMetres(estimate.risk)
end

---Once the estimates of a choice have come, logs them, keeps the best reachable candidate and gives the
---MoveTo to it. A choice with no reachable candidate gives no order.
---@param ship SandboxShip
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
---@param simTime number
---@param metrics ScenarioMetrics
local function finishChoice(ship, ships, obstacles, simTime, metrics)
    local b = ship.behaviour
    local estimates = Nav.takeEstimates(b.choosing.request)
    if not estimates then return end
    local index = b.choosing.index
    b.choosing = nil
    local order = b.spec.orders[index]
    local weight, best = order.choose.riskWeightSPerM, nil
    for i, estimate in ipairs(estimates) do
        if estimate.reachable then
            local score = scoreOf(estimate, weight)
            metrics:log("%s: candidate %d: time %.1f s, path %.0f m, risk %.0f m, score %.1f", ship.label, i,
                estimate.time, Metres.toMetres(estimate.distance), Metres.toMetres(estimate.risk), score)
            if not best or score < best.score then best = { index = i, score = score, time = estimate.time } end
        else
            metrics:log("%s: candidate %d: not reachable", ship.label, i)
        end
    end
    if not best then
        metrics:log("%s: no candidate is reachable", ship.label)
        return
    end
    metrics:log("%s: chose candidate %d", ship.label, best.index)
    b.chosen, b.chosenPoint = best.index, order.choose.points[best.index]
    b.chosenEstimate = { time = best.time, givenAt = simTime }
    giveOrder(ship, index, ships, obstacles, { kind = "MoveTo", point = b.chosenPoint, priority = order.priority })
end

---Gives the order at `index` of the ship's spec, or, for an order that chooses, asks for the estimates.
---@param ship SandboxShip
---@param index integer
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
local function dispatchOrder(ship, index, ships, obstacles)
    if ship.behaviour.spec.orders[index].choose then
        startChoice(ship, index)
    else
        giveOrder(ship, index, ships, obstacles)
    end
end

---Gives a ship the orders of its spec that are due: those with no `afterS`, then those whose time has come.
---Orders wait for a choice that is not made yet.
---@param ship SandboxShip
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
---@param simTime number Seconds of simulated time since the scenario started
---@param metrics ScenarioMetrics
function ShipBehaviour.issueDueOrders(ship, ships, obstacles, simTime, metrics)
    local b = ship.behaviour
    if b.choosing then finishChoice(ship, ships, obstacles, simTime, metrics) end
    if b.choosing then return end
    local later = {}
    for _, index in ipairs(b.pending) do
        if b.spec.orders[index].afterS <= simTime and not b.choosing then
            dispatchOrder(ship, index, ships, obstacles)
        else
            table.insert(later, index)
        end
    end
    b.pending = later
end

---What a ship that chose its point estimated for it and when it was given the order, for a line in the log.
---@param ship SandboxShip
---@return { time: number, givenAt: number }|nil
---@return boolean|nil suspended Whether the order was suspended on the way, which the flown time includes
function ShipBehaviour.choiceReport(ship)
    local b = ship.behaviour
    return b and b.chosenEstimate, b and b.wasSuspended
end

---Gives a ship its orders, and the rule it is judged by what that rule needs of the last one.
---@param ship SandboxShip
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
local function startOrders(ship, ships, obstacles)
    local b, spec = ship.behaviour, ship.behaviour.spec
    b.orders, b.pending = {}, {}
    if ship.spec.expect and ship.spec.expect.zone then b.clearance, b.zoneClearance = {}, {} end
    for index, order in ipairs(spec.orders) do
        if order.afterS then
            table.insert(b.pending, index)
        else
            dispatchOrder(ship, index, ships, obstacles)
        end
    end
    if b.kind == "Orbit" then
        b.radius = Metres.length(spec.radiusM)
        b.normal = spec.normal and unitOr({ x = spec.normal[1], y = spec.normal[2], z = spec.normal[3] }, 0, nil)
    elseif b.kind == "Follow" then
        b.offset = { x = Metres.length(spec.offsetM[1]), y = Metres.length(spec.offsetM[2]),
            z = Metres.length(spec.offsetM[3]) }
    end
end

---Looks up the bodies the behaviour is about, and gives the ship its first order, or all its orders.
---@param nav NavWorld
---@param ship SandboxShip
---@param ships SandboxShip[]
---@param obstacles SandboxObstacle[]
---@param rules SandboxJudgingRules
---@param planner PlanClient|nil Needed by a Plan ship
function ShipBehaviour.start(nav, ship, ships, obstacles, rules, planner)
    local b, spec = ship.behaviour, ship.behaviour.spec
    b.topSpeed = ConstructComponents.FlightProfile.ProfileOf(ship.entity).maxSpeed
    b.tracked = spec.obstacle and obstacles[spec.obstacle] or spec.ship and ships[spec.ship]
    if spec.orders then
        startOrders(ship, ships, obstacles)
    elseif b.kind == "Plan" then
        -- Flies at the goal while the plan is made; the plan then replaces the course.
        assert(planner, "ShipBehaviour: a Plan ship needs a planner")
        b.clearance, b.zoneClearance, b.obstacles = {}, {}, obstacles
        b.plansTaken, b.replans = 0, 0
        b.goal = { frame = ship.targetFrame, offset = ship.targetFrame and ship.targetOffset or ship.target }
        b.passRadius = Metres.length(spec.passRadiusM)
        requestPlan(nav, ship, rules, planner, 0)
    elseif b.kind == "Seek" then
        b.points, b.pointIndex, b.phase = {}, 1, "pass"
        for _, point in ipairs(spec.through) do table.insert(b.points, Metres.position(point)) end
        b.passRadius = Metres.length(spec.passRadiusM)
        if spec.passSpeedShare then b.passSpeed = spec.passSpeedShare * b.topSpeed end
        if spec.speedCapShare then
            b.speedCap = spec.speedCapShare * b.topSpeed
            nav:setSpeedCap(ship.agent, b.speedCap)
        end
        if spec.measureTurn then b.turn = { radius = math.huge, speed = 0 } end
        ship.target = b.points[1]
        seekPoint(nav, ship)
    elseif b.kind == "Hold" then
        b.phase = "cruise"
        nav:setArrive(ship.agent, ship.target, rules.range)
    elseif b.kind == "Pursue" then
        b.standOff = Metres.length(spec.standOffM)
        nav:setPursue(ship.agent, b.tracked.rb, b.standOff, rules.range)
    elseif b.kind == "Follow" then
        local right, up, forward = Metres.length(spec.offsetM[1]), Metres.length(spec.offsetM[2]),
            Metres.length(spec.offsetM[3])
        b.offset = { x = right, y = up, z = forward }
        nav:setFollow(ship.agent, b.tracked.rb, right, up, forward, rules.range)
    elseif b.kind == "Orbit" then
        local n = spec.normal
        b.normal = scale({ x = n[1], y = n[2], z = n[3] }, 1 / length({ x = n[1], y = n[2], z = n[3] }))
        b.radius = Metres.length(spec.radiusM)
        nav:setOrbit(ship.agent, b.tracked.rb, b.radius, Vec3d(n[1], n[2], n[3]), rules.range)
    else
        error("ShipBehaviour: unknown kind " .. tostring(b.kind))
    end
    ShipBehaviour.refreshTarget(ship)
end

---What is wrong with the speed a Seek ship passed a point at: too slow for a ship that does not
---brake for it, too fast for one that does or is capped.
---@param b SandboxBehaviour
---@param speed number Game units per second
---@return string|nil
local function passProblem(b, speed)
    local limit = math.min(b.passSpeed, b.speedCap or math.huge)
    if limit == math.huge then
        return speed < PASS_SPEED_FRACTION * b.topSpeed and "too slow" or nil
    end
    return speed > limit * (1 + PASS_SPEED_TOLERANCE) and "too fast" or nil
end

---Notes how tightly the ship turns: the smallest radius it has flown, speed over the rate its
---velocity turns at, between two steps. Called after each navigation step.
---@param b SandboxBehaviour
---@param ship SandboxShip
---@param speed number Game units per second
---@param simTime number
local function trackTurn(b, ship, speed, simTime)
    local turn = b.turn
    local velocity = ship.rb:getVelocity()
    local previous = turn.velocity
    local before = previous and length(previous) or 0
    if before > 0 and speed >= MIN_TURN_SPEED_FRACTION * b.topSpeed and simTime > turn.time then
        local cosine = dot(previous, velocity) / (before * speed)
        local angle = math.acos(math.max(-1, math.min(1, cosine)))
        if angle > MIN_TURN_ANGLE then
            local radius = speed / (angle / (simTime - turn.time))
            if radius < turn.radius then turn.radius, turn.speed = radius, speed end
        end
    end
    turn.velocity, turn.time = { x = velocity.x, y = velocity.y, z = velocity.z }, simTime
end

---The share of its forward thrust a ship needs to fly a circle of `radius` at `speed`. With forward
---acceleration `a` and linear drag `k` that is the centripetal `s²/r` across and `k·s` along the
---velocity: `√(s⁴/r² + k²s²) / a`, the quantity the Orbit behaviour holds at a named constant.
---@param ship SandboxShip
---@param speed number Game units per second
---@param radius number Game units
---@return number
local function turnShare(ship, speed, radius)
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    local k = profile.linearDrag
    return math.sqrt(speed ^ 4 / (radius * radius) + k * k * speed * speed) / (profile.thrustForward / profile.mass)
end

---What a Plan ship's follower reports to: the scenario's metrics, the ship's progress watch, and
---the move on to arriving at the goal after the last corner. `nav`, `rules` and `metrics` are the
---current ones, set by `advance`.
---@param ship SandboxShip
---@return table
local function planListener(ship)
    local listener = { ship = ship }
    function listener:pointPassed(index, speed, simTime)
        self.metrics:planPointPassed(self.ship, index, speed, simTime)
    end
    function listener:arcFlown(flight, simTime)
        self.metrics:planArcFlown(self.ship, flight, simTime)
    end
    function listener:legChanged(simTime)
        ProgressWatch.reset(self.ship.watch, simTime)
    end
    function listener:finished(_)
        self.ship.behaviour.phase = "arrive"
        arriveAtGoal(self.nav, self.ship, self.rules.range)
    end
    return listener
end

---Flies a Plan ship along its path through the shared follower.
---@param nav NavWorld
---@param ship SandboxShip
---@param arrived boolean The `NavWorld`'s arrival flag for the ship
---@param speed number Game units per second
---@param simTime number
---@param rules SandboxJudgingRules
---@param metrics ScenarioMetrics
local function advancePlan(nav, ship, arrived, speed, simTime, rules, metrics)
    local b = ship.behaviour
    local listener = b.listener
    listener.nav, listener.rules, listener.metrics = nav, rules, metrics
    b.follower:advance(arrived, speed, simTime, listener)
    if b.phase == "pass" then ship.target = b.follower.target end
end

---Moves a Seek or Plan ship on past a point it has passed, flies a Plan ship along its path, and
---switches a Hold ship to Hold once it is flying fast enough. Called after each navigation step.
---@param nav NavWorld
---@param ship SandboxShip
---@param arrived boolean The `NavWorld`'s arrival flag for the ship
---@param simTime number
---@param rules SandboxJudgingRules
---@param metrics ScenarioMetrics
function ShipBehaviour.advance(nav, ship, arrived, simTime, rules, metrics)
    local b = ship.behaviour
    local speed = ship.rb:getSpeed()
    b.peakSpeed = math.max(b.peakSpeed, speed)
    if b.turn then trackTurn(b, ship, speed, simTime) end
    if b.kind == "Plan" then
        if b.phase == "pass" then advancePlan(nav, ship, arrived, speed, simTime, rules, metrics) end
    elseif b.kind == "Seek" and b.phase == "pass" and arrived then
        metrics:seekPassed(ship, b.pointIndex, speed, passProblem(b, speed), simTime)
        b.pointIndex = b.pointIndex + 1
        if not b.points[b.pointIndex] then
            b.phase = "arrive"
            ship.target = Metres.position(ship.spec.target)
            nav:setArrive(ship.agent, ship.target, rules.range)
            nav:refresh(ship.agent)
        else
            ship.target = b.points[b.pointIndex]
            seekPoint(nav, ship)
        end
        ProgressWatch.reset(ship.watch, simTime)
    elseif b.kind == "Hold" and b.phase == "cruise" and speed >= b.spec.holdAtSpeed * b.topSpeed then
        local at = ship.rb:getPos()
        ship.target = Position(at.x, at.y, at.z)
        b.phase = "switched"
        nav:setHold(ship.agent, rules.range)
        nav:refresh(ship.agent)
        metrics:log("%s holds at %.0f m/s, %.2f s", ship.label, Metres.toMetres(speed), simTime)
    end
end

---The obstacles as the frames of the plan being flown: each has the body (`rb`) and where it is at
---a planned time, which counts from the moment the plan was requested.
---@param b SandboxBehaviour
---@return PlanFrame[]
local function planFrames(b)
    local frames = {}
    for i, obstacle in ipairs(b.obstacles) do
        frames[i] = { rb = obstacle.rb, positionAt = function(t) return obstacle.positionAt(b.planTime + t) end }
    end
    return frames
end

---Whether a Plan ship's plan was made under a cost profile that has since changed.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.planIsStale(ship)
    local b = ship.behaviour
    return b.kind == "Plan" and b.revision ~= costRevision(ship)
end

---Drops a Plan ship's plan, or the request for it, and asks for a new one from where the ship is.
---The ship flies at the goal until it comes.
---@param nav NavWorld
---@param ship SandboxShip
---@param rules SandboxJudgingRules
---@param planner PlanClient
---@param simTime number
function ShipBehaviour.replan(nav, ship, rules, planner, simTime)
    local b = ship.behaviour
    planner:discardFor(ship)
    b.follower, b.listener = nil, nil
    nav:setSpeedCap(ship.agent, math.huge)
    b.replans = b.replans + 1
    requestPlan(nav, ship, rules, planner, simTime)
    ProgressWatch.reset(ship.watch, simTime)
end

---Takes a Plan ship's plan: the ship passes the plan's waypoints, the goal last being where it
---stops. A plan that is not Ok or has no detour leaves the ship flying at the goal.
---@param nav NavWorld
---@param ship SandboxShip
---@param plan Plan
---@param simTime number
function ShipBehaviour.planReceived(nav, ship, plan, simTime)
    local b = ship.behaviour
    b.plansTaken = b.plansTaken + 1
    local points = {}
    if plan.status == SystemPlanner.Status.Ok then
        for i = 1, #plan.waypoints - 1 do points[i] = plan.waypoints[i] end
    end
    if #points == 0 then
        b.phase = "arrive"
        return
    end
    b.goal.time = plan.waypoints[#plan.waypoints].time
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    b.follower = PlanFollower.new({
        nav = nav, agent = ship.agent, rb = ship.rb, frames = planFrames(b), points = points, goal = b.goal,
        start = b.startPoint, passRadius = b.passRadius, accel = profile.thrustForward / profile.mass,
        drag = profile.linearDrag,
    })
    b.listener = planListener(ship)
    b.phase = "pass"
    ship.target = b.follower.target
    ProgressWatch.reset(ship.watch, simTime)
end

---Notes how close a ship that keeps clearances (a Plan ship, or an Orders ship that expects a zone) is to each
---obstacle's safe sphere and to each cost zone's sphere.
---Called after a physics step.
---@param ship SandboxShip
---@param obstacles SandboxObstacle[]
---@param zones SandboxZone[]
function ShipBehaviour.trackClearance(ship, obstacles, zones)
    local b = ship.behaviour
    if not b or not b.zoneClearance then return end
    local at = ship.rb:getPos()
    for i, obstacle in ipairs(obstacles) do
        local clearance = at:distance(obstacle.rb:getPos()) - obstacle.safeRadius
        b.clearance[i] = math.min(b.clearance[i] or math.huge, clearance)
    end
    for i, zone in ipairs(zones) do
        local clearance = at:distance(zone.centre) - zone.radius
        b.zoneClearance[i] = math.min(b.zoneClearance[i] or math.huge, clearance)
    end
end

---Whether a ship at `distance` from its point is there and, if `relativeTo` is a body, at rest
---relative to it.
local function isThere(ship, distance, rules, relativeTo)
    local speed = relativeTo
        and length(sub(ship.rb:getVelocity(), relativeTo.rb:getVelocity()))
        or ship.rb:getSpeed()
    return distance <= rules.range and speed <= rules.speed
end

---Whether the ship has stayed on station for `STATION_HOLD_S` in a row.
local function heldOnStation(b, on, simTime)
    if not on then
        b.onSince = nil
        return false
    end
    b.onSince = b.onSince or simTime
    return simTime - b.onSince >= STATION_HOLD_S
end

---Pursue and Follow: held on station, at rest relative to the target. A follower is judged once
---its leader has finished its own flight.
local function judgeStation(b, ship, distance, simTime, rules)
    local on = isThere(ship, distance, rules, b.tracked)
    b.onStation = on
    if distance <= rules.range then b.engaged = true end
    if b.engaged then b.maxError = math.max(b.maxError, distance) end
    local ready = b.kind ~= "Follow" or b.tracked.state ~= "flying"
    local judged = on and ready
    b.turnRate = judged and math.max(b.turnRate or 0, length(ship.rb:getVelocityA())) or 0
    return heldOnStation(b, judged, simTime)
end

---Whether a ship that tracks a moving point is on station now: within range of its point and at
---rest relative to the tracked body. It is judged, and can arrive, only after it has held that.
---@param ship SandboxShip
---@return boolean
function ShipBehaviour.isOnStation(ship)
    local b = ship.behaviour
    return b ~= nil and b.onStation == true
end

---Orbit: one full turn about the centre, anticlockwise about the normal, kept within the orbit
---tolerance of the circle. Leaving the circle starts the turn again.
local function judgeOrbit(b, ship, distance, rules)
    -- The plane an order chooses is not known until the ship is on the circle.
    if not b.normal then return false end
    if distance > math.max(rules.range, ORBIT_TOLERANCE * b.radius) then
        b.turned, b.previous = 0, nil
        if b.engaged then b.maxError = math.max(b.maxError, distance) end
        return false
    end
    b.engaged = true
    b.maxError = math.max(b.maxError, distance)
    local offset = sub(ship.rb:getPos(), b.tracked.rb:getPos())
    local planar = sub(offset, scale(b.normal, dot(offset, b.normal)))
    if b.previous then
        b.turned = b.turned + math.atan2(dot(b.normal, cross(b.previous, planar)), dot(b.previous, planar))
    end
    b.previous = planar
    return b.turned >= b.turnLimit
end

---A ship flown by orders whose last order is a Hold: held in place, at rest, for `STATION_HOLD_S`
---from where it was when the Hold began.
local function judgeOrderHold(b, ship, distance, simTime, rules)
    if not b.holdPoint then return false end
    return heldOnStation(b, isThere(ship, distance, rules), simTime)
end

---Whether the ship has done what its behaviour asks, judged after a physics step. Also moves a Hold
---ship from "switched" to "returning" once it has left its point.
---@param ship SandboxShip
---@param simTime number
---@param rules SandboxJudgingRules
---@return boolean
function ShipBehaviour.judge(ship, simTime, rules)
    local b = ship.behaviour
    local distance = ship.rb:getPos():distance(ship.target)
    if b.kind == "Pursue" or b.kind == "Follow" then
        return judgeStation(b, ship, distance, simTime, rules)
    elseif b.kind == "Orbit" then
        return judgeOrbit(b, ship, distance, rules)
    elseif b.kind == "OrderDone" then
        local order = b.judgedOrder
        if not order then return false end
        local status = Nav.status(order)
        if status == Enums.NavOrderStatus.Suspended then b.wasSuspended = true end        return status == Enums.NavOrderStatus.Done and (not b.spec.suspends or b.wasSuspended == true)
    elseif b.kind == "OrderHold" then
        return judgeOrderHold(b, ship, distance, simTime, rules)
    elseif b.kind == "Seek" or b.kind == "Plan" then
        local frame = b.goal and b.goal.frame and b.obstacles[b.goal.frame]
        return b.phase == "arrive" and isThere(ship, distance, rules, frame)
    end
    -- Hold: the ship must leave its point and come back to it.
    if b.phase == "switched" and distance > rules.range then b.phase = "returning" end
    return b.phase == "returning" and isThere(ship, distance, rules)
end

---The speed cap a ship has been flown over: the cap and the largest speed, or nil for a ship
---without a cap or one that stayed within it.
---@param ship SandboxShip
---@return number|nil cap Game units per second
---@return number|nil peak Game units per second
function ShipBehaviour.capExceeded(ship)
    local b = ship.behaviour
    if b and b.speedCap and b.peakSpeed > b.speedCap * (1 + CAP_TOLERANCE) then
        return b.speedCap, b.peakSpeed
    end
    return nil
end

---A line for the summary: a capped ship's largest speed against its cap.
---@param ship SandboxShip
---@return number|nil cap Game units per second, or nil for a ship without a cap
---@return number|nil peak Game units per second
function ShipBehaviour.capReport(ship)
    local b = ship.behaviour
    if b and b.speedCap then return b.speedCap, b.peakSpeed end
    return nil
end

---A line for the summary: the tightest turn a ship with `measureTurn` flew, and the share of its
---forward thrust the turn needed (see `turnShare`).
---@param ship SandboxShip
---@return number|nil radius Game units, or nil for a ship that does not measure or never turned
---@return number|nil speed Game units per second at that radius
---@return number|nil share Fraction of the forward thrust
function ShipBehaviour.turnReport(ship)
    local b = ship.behaviour
    if not (b and b.turn) or b.turn.radius == math.huge then return nil end
    local s, r = b.turn.speed, b.turn.radius
    return r, s, turnShare(ship, s, r)
end

---A line for the summary: how fast a Pursue or Follow ship turned while it was held on station.
---@param ship SandboxShip
---@return number|nil turnRate Largest turn rate, in radians per second, or nil for the other kinds
function ShipBehaviour.stationTurnRate(ship)
    local b = ship.behaviour
    return b and (b.kind == "Pursue" or b.kind == "Follow") and b.engaged and b.turnRate or nil
end

---A line for the summary: how well a tracking ship kept its station, or nil for the other kinds.
---@param ship SandboxShip
---@return number|nil maxError The largest distance from the station point since it was reached, in game units
function ShipBehaviour.maxStationError(ship)
    local b = ship.behaviour
    return b and TRACKING[b.kind] and b.engaged and b.maxError or nil
end

return ShipBehaviour
