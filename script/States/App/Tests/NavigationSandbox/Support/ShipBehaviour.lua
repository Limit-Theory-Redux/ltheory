-- The behaviours a sandbox ship can be given besides flying to a target (Seek, Hold, Pursue,
-- Follow, Orbit): issuing each one to the `NavWorld`, the point the ship is to be at, and the
-- verdict on whether it has done what the behaviour asks. The geometry here is the sandbox's own,
-- written from the behaviour's definition, so it checks the engine's maths instead of repeating it.
-- Design: doc/script/navigation-sandbox.md
local Metres              = require("States.App.Tests.NavigationSandbox.Support.Metres")
local ConstructComponents = require("Modules.Constructs.Components")
local SystemPlanner       = require("Shared.Helpers.SystemPlanner")

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
-- A ship on an arc is steered at the point of the arc it would reach in this many seconds at the
-- corner's speed, at most this many radians ahead of it (pure pursuit).
local ARC_LOOKAHEAD_S        = 0.3
local MAX_ARC_LOOKAHEAD      = 0.6
-- An orbiting ship has orbited after it has turned this far about the centre.
local FULL_TURN           = 2 * math.pi
-- An orbiting ship is on its circle within this fraction of the radius: the wanted velocity is
-- the circle's tangent, and a ship at kilometres per second runs a little wide of it.
local ORBIT_TOLERANCE     = 0.1
-- A direction is taken from a vector at least this long, relative to the radius it is compared with.
local MIN_RELATIVE_LENGTH = 1e-6

---@class SandboxBehaviourSpec
---@field kind "Seek"|"Hold"|"Pursue"|"Follow"|"Orbit"|"Plan"
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
---@field kind string
---@field spec SandboxBehaviourSpec
---@field tracked table|nil The sandbox ship or obstacle the behaviour is about
---@field topSpeed number Game units per second
---@field engaged boolean Whether the ship has been on station; deadlocks are judged only before
---@field maxError number Largest distance from the station point since it was engaged, in game units
---@field phase string|nil Seek: "pass" or "arrive". Plan: "waiting" for the plan, then as Seek. Hold: "cruise", "switched" or "returning"
---@field points Position[]|SandboxWaypoint[]|nil Seek: the points to pass. Plan: the plan's waypoints before the goal
---@field pointIndex integer|nil Seek, Plan: the point being flown to
---@field passRadius number|nil Seek, Plan: game units
---@field passSpeed number Seek, Plan: game units per second a point is passed at; `math.huge` for top speed
---@field speedCap number|nil Seek: game units per second, as given to the `NavWorld`
---@field peakSpeed number Seek: the largest speed the ship has flown at, in game units per second
---@field turn SandboxTurn|nil Seek with `measureTurn`: the tightest turn so far
---@field startPoint table|nil Plan: { x, y, z } where the ship was when it asked for the plan, the point before the first corner
---@field arc SandboxArcFlight|nil Plan: the arc being flown; nil on a leg
---@field goal SandboxWaypoint|nil Plan: where the ship stops; with a frame, a point that rides on that obstacle
---@field obstacles SandboxObstacle[]|nil Plan: the scenario's obstacles, which waypoint frames index
---@field clearance table<integer, number>|nil Plan: per obstacle index, the smallest distance from the ship's centre to the obstacle's safe sphere since the start, in game units
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

---@class SandboxArc
---@field entry table { x, y, z }
---@field exit table { x, y, z }
---@field centre table { x, y, z }
---@field normal table Unit { x, y, z }: the ship turns anticlockwise about it
---@field turn number Radians from the entry to the exit
---@field radius number Game units

---What a Plan ship did on one arc, for the log.
---@class SandboxArcFlight
---@field index integer Index of the corner in the plan
---@field radius number Game units, as planned
---@field speed number Game units per second, as planned
---@field enteredAt number Simulated time the ship reached the entry
---@field entrySpeed number Game units per second
---@field peakSpeed number Largest speed on the arc, game units per second
---@field maxError number Largest distance from the arc's circle, game units
---@field maxShare number Largest share of the forward thrust the planned radius needs at the speed flown

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

---@param spec SandboxBehaviourSpec
---@return SandboxBehaviour
function ShipBehaviour.create(spec)
    return {
        kind = spec.kind, spec = spec, engaged = false, maxError = 0, turned = 0, topSpeed = 0,
        passSpeed = math.huge, peakSpeed = 0,
    }
end

---Sends a Seek or Plan ship through its current point.
---@param nav NavWorld
---@param ship SandboxShip
local function seekPoint(nav, ship)
    local b = ship.behaviour
    nav:setSeek(ship.agent, ship.target, b.passRadius, b.passSpeed)
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
    -- Orbit: the nearest point of the circle.
    local offset = sub(shipPos, trackedPos)
    local planar = sub(offset, scale(b.normal, dot(offset, b.normal)))
    local direction = unitOr(planar, b.radius * MIN_RELATIVE_LENGTH, anyAcross(b.normal))
    return add(trackedPos, scale(direction, b.radius))
end

local TRACKING = { Pursue = true, Follow = true, Orbit = true }

---The world point of a plan waypoint now: its offset from its frame's obstacle, or the point itself.
---@param b SandboxBehaviour
---@param waypoint SandboxWaypoint
---@return Position
local function pointNow(b, waypoint)
    local offset = waypoint.offset
    if not waypoint.frame then return offset end
    local centre = b.obstacles[waypoint.frame].rb:getPos()
    return Position(centre.x + offset.x, centre.y + offset.y, centre.z + offset.z)
end

---A plan waypoint's world point at its planned time, which is where the kernel placed it.
---@param b SandboxBehaviour
---@param waypoint SandboxWaypoint
---@return Position
local function pointPlanned(b, waypoint)
    local offset = waypoint.offset
    if not waypoint.frame then return offset end
    local centre = b.obstacles[waypoint.frame].positionAt(waypoint.time)
    return Position(centre.x + offset.x, centre.y + offset.y, centre.z + offset.z)
end

---The arc of the corner a Plan ship is flying to. The corner is where it is now; its neighbours (the
---start for the first corner, the goal for the last) keep the planned offset from it, so the arc is
---the one the kernel cleared even when the corner or a neighbour rides on a moving body.
---@param b SandboxBehaviour
---@return SandboxArc|nil arc Nil for a sharp corner
local function currentArc(b)
    local index = b.pointIndex
    local waypoint = b.points[index]
    if waypoint.radius <= 0 then return nil end
    local corner = pointNow(b, waypoint)
    local plannedCorner = pointPlanned(b, waypoint)
    local function around(planned)
        return Position(corner.x + planned.x - plannedCorner.x, corner.y + planned.y - plannedCorner.y,
            corner.z + planned.z - plannedCorner.z)
    end
    local before = around(index == 1 and toPosition(b.startPoint) or pointPlanned(b, b.points[index - 1]))
    local after = around(pointPlanned(b, b.points[index + 1] or b.goal))
    local ex, ey, ez, xx, xy, xz, ox, oy, oz, kx, ky, kz, turn = SystemPlanner.cornerArc(
        before.x, before.y, before.z, corner.x, corner.y, corner.z, after.x, after.y, after.z, waypoint.radius)
    if not ex then return nil end
    return {
        entry = { x = ex, y = ey, z = ez }, exit = { x = xx, y = xy, z = xz },
        centre = { x = ox, y = oy, z = oz }, normal = { x = kx, y = ky, z = kz },
        turn = turn, radius = waypoint.radius,
    }
end

---The point a Plan ship is to be at now, which is what its progress is judged against: the entry
---of the arc or the sharp corner it is flying to, the end of the arc it is on, then the goal.
---@param b SandboxBehaviour
---@return Position
local function planPoint(b)
    if b.phase ~= "pass" then return pointNow(b, b.goal) end
    local arc = currentArc(b)
    if not arc then return pointNow(b, b.points[b.pointIndex]) end
    return toPosition(b.arc and arc.exit or arc.entry)
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
end

---Sets `ship.target` to the point a tracking or Plan ship is to be at, which moves with its body;
---other ships keep theirs.
---@param ship SandboxShip
function ShipBehaviour.refreshTarget(ship)
    local b = ship.behaviour
    if not b then return end
    if TRACKING[b.kind] then
        ship.target = toPosition(stationPoint(b, ship))
    elseif b.kind == "Plan" then
        ship.target = planPoint(b)
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
    return (TRACKING[b.kind] and b.engaged) or (b.kind == "Hold" and b.phase ~= "cruise")
end

---Looks up the bodies the behaviour is about, and gives the ship its first order.
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
    if b.kind == "Plan" then
        -- Flies at the goal while the plan is made; the plan then replaces the course.
        assert(planner, "ShipBehaviour: a Plan ship needs a planner")
        b.phase, b.clearance, b.obstacles = "waiting", {}, obstacles
        b.goal = { frame = ship.targetFrame, offset = ship.targetFrame and ship.targetOffset or ship.target }
        b.passRadius = Metres.length(spec.passRadiusM)
        arriveAtGoal(nav, ship, rules.range)
        local at = ship.rb:getPos()
        b.startPoint = { x = at.x, y = at.y, z = at.z }
        planner:request(ship, b.goal.offset, b.goal.frame, b.topSpeed, obstacles, 0)
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

---Steps a Plan ship on to the corner after the one it has passed, or to its goal after the last.
---@param nav NavWorld
---@param ship SandboxShip
---@param rules SandboxJudgingRules
---@param simTime number
local function nextPlanPoint(nav, ship, rules, simTime)
    local b = ship.behaviour
    b.pointIndex = b.pointIndex + 1
    if not b.points[b.pointIndex] then
        b.phase = "arrive"
        arriveAtGoal(nav, ship, rules.range)
    end
    ship.bestDistance, ship.progressTime = math.huge, simTime
end

---Starts the arc of the corner the ship has reached the entry of: it is held to the corner's speed.
---@param nav NavWorld
---@param ship SandboxShip
---@param speed number Game units per second now
---@param simTime number
local function beginArc(nav, ship, speed, simTime)
    local b = ship.behaviour
    local waypoint = b.points[b.pointIndex]
    b.arc = {
        index = b.pointIndex, radius = waypoint.radius, speed = waypoint.speed, enteredAt = simTime,
        entrySpeed = speed, peakSpeed = speed, maxError = 0, maxShare = 0,
    }
    nav:setSpeedCap(ship.agent, waypoint.speed)
end

---Where the ship is on `arc`: the angle about the centre from the entry, and its distance from
---the arc's circle.
---@param arc SandboxArc
---@param position table { x, y, z }
---@return number angle Radians, positive toward the exit
---@return number error Game units
local function arcPosition(arc, position)
    local fromCentre = sub(position, arc.centre)
    local fromEntry = sub(arc.entry, arc.centre)
    local angle = math.atan2(dot(arc.normal, cross(fromEntry, fromCentre)), dot(fromEntry, fromCentre))
    local across = dot(arc.normal, fromCentre)
    local inPlane = length(sub(fromCentre, scale(arc.normal, across)))
    return angle, math.sqrt((inPlane - arc.radius) ^ 2 + across ^ 2)
end

---Steers a ship on `arc` at the point of it a short way ahead (pure pursuit), and keeps what the
---arc asks of the controller. The arc is over when the ship has swept its angle.
---@param nav NavWorld
---@param ship SandboxShip
---@param arc SandboxArc
---@param speed number Game units per second
---@return boolean onArc False once the ship is past the exit
local function flyArc(nav, ship, arc, speed)
    local b = ship.behaviour
    local flight = b.arc
    local angle, error = arcPosition(arc, ship.rb:getPos())
    flight.peakSpeed = math.max(flight.peakSpeed, speed)
    flight.maxError = math.max(flight.maxError, error)
    flight.maxShare = math.max(flight.maxShare, turnShare(ship, speed, arc.radius))
    if angle >= arc.turn then return false end
    local ahead = math.min(MAX_ARC_LOOKAHEAD, flight.speed * ARC_LOOKAHEAD_S / arc.radius)
    local x, y, z = SystemPlanner.pointOnArc(arc.centre.x, arc.centre.y, arc.centre.z,
        arc.entry.x, arc.entry.y, arc.entry.z, arc.normal.x, arc.normal.y, arc.normal.z,
        math.min(arc.turn, math.max(angle, 0) + ahead))
    nav:setSeek(ship.agent, Position(x, y, z), b.passRadius, math.huge)
    return true
end

---Ends the arc: the speed cap is lifted, and the ship goes on to the next corner.
---@param nav NavWorld
---@param ship SandboxShip
---@param rules SandboxJudgingRules
---@param simTime number
---@param metrics ScenarioMetrics
local function endArc(nav, ship, rules, simTime, metrics)
    local b = ship.behaviour
    nav:setSpeedCap(ship.agent, math.huge)
    metrics:planArcFlown(ship, b.arc, simTime)
    b.arc = nil
    nextPlanPoint(nav, ship, rules, simTime)
end

---Sends a Plan ship along its leg to the entry of `arc`, or to the sharp corner when there is
---none, to pass it at the corner's speed.
---@param nav NavWorld
---@param ship SandboxShip
---@param arc SandboxArc|nil The arc of the corner being flown to
local function seekPlanLeg(nav, ship, arc)
    local b = ship.behaviour
    ship.target = arc and toPosition(arc.entry) or pointNow(b, b.points[b.pointIndex])
    nav:setSeek(ship.agent, ship.target, b.passRadius, b.points[b.pointIndex].speed)
end

---Flies a Plan ship along its path. On a leg it is sent to the arc's entry (or the sharp corner)
---at the corner's speed, the points taken from where their bodies are now; on reaching it, it
---takes the arc, then the next leg.
---@param nav NavWorld
---@param ship SandboxShip
---@param arrived boolean The `NavWorld`'s arrival flag for the ship
---@param speed number Game units per second
---@param simTime number
---@param rules SandboxJudgingRules
---@param metrics ScenarioMetrics
local function advancePlan(nav, ship, arrived, speed, simTime, rules, metrics)
    local b = ship.behaviour
    local arc = currentArc(b)
    if b.arc then
        if arc and flyArc(nav, ship, arc, speed) then
            ship.target = toPosition(arc.exit)
            return
        end
        endArc(nav, ship, rules, simTime, metrics)
        if b.phase ~= "pass" then return end
        arc = currentArc(b)
    elseif arrived then
        metrics:planPointPassed(ship, b.pointIndex, speed, simTime)
        if arc then
            beginArc(nav, ship, speed, simTime)
            if flyArc(nav, ship, arc, speed) then
                ship.target = toPosition(arc.exit)
                return
            end
            endArc(nav, ship, rules, simTime, metrics)
        else
            nextPlanPoint(nav, ship, rules, simTime)
        end
        if b.phase ~= "pass" then return end
        arc = currentArc(b)
    end
    seekPlanLeg(nav, ship, arc)
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
        else
            ship.target = b.points[b.pointIndex]
            seekPoint(nav, ship)
        end
        ship.bestDistance, ship.progressTime = math.huge, simTime
    elseif b.kind == "Hold" and b.phase == "cruise" and speed >= b.spec.holdAtSpeed * b.topSpeed then
        local at = ship.rb:getPos()
        ship.target = Position(at.x, at.y, at.z)
        b.phase = "switched"
        nav:setHold(ship.agent, rules.range)
        metrics:log("%s holds at %.0f m/s, %.2f s", ship.label, Metres.toMetres(speed), simTime)
    end
end

---Takes a Plan ship's plan: the ship passes the plan's waypoints, the goal last being where it
---stops. A plan that is not Ok or has no detour leaves the ship flying at the goal.
---@param nav NavWorld
---@param ship SandboxShip
---@param plan SandboxPlan
---@param simTime number
function ShipBehaviour.planReceived(nav, ship, plan, simTime)
    local b = ship.behaviour
    local points = {}
    if plan.status == SystemPlanner.Status.Ok then
        for i = 1, #plan.waypoints - 1 do points[i] = plan.waypoints[i] end
    end
    if #points == 0 then
        b.phase = "arrive"
        return
    end
    b.goal.time = plan.waypoints[#plan.waypoints].time
    b.points, b.pointIndex, b.phase = points, 1, "pass"
    seekPlanLeg(nav, ship, currentArc(b))
    ship.bestDistance, ship.progressTime = math.huge, simTime
end

---Notes how close a Plan ship is to each obstacle's safe sphere. Called after a physics step.
---@param ship SandboxShip
---@param obstacles SandboxObstacle[]
function ShipBehaviour.trackClearance(ship, obstacles)
    local b = ship.behaviour
    if not b or b.kind ~= "Plan" then return end
    local at = ship.rb:getPos()
    for i, obstacle in ipairs(obstacles) do
        local clearance = at:distance(obstacle.rb:getPos()) - obstacle.safeRadius
        b.clearance[i] = math.min(b.clearance[i] or math.huge, clearance)
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
    return b.turned >= FULL_TURN
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
