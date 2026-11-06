-- The behaviours a sandbox ship can be given besides flying to a target (Seek, Hold, Pursue,
-- Follow, Orbit): issuing each one to the `NavWorld`, the point the ship is to be at, and the
-- verdict on whether it has done what the behaviour asks. The geometry here is the sandbox's own,
-- written from the behaviour's definition, so it checks the engine's maths instead of repeating it.
-- Design: doc/script/navigation-sandbox.md
local Metres              = require("States.App.Tests.NavigationSandbox.Support.Metres")
local ConstructComponents = require("Modules.Constructs.Components")

-- A ship that tracks a moving point has done so when it has stayed on station this long.
local STATION_HOLD_S      = 5
-- A Seek point is passed at least this fraction of top speed: Arrive would be braking there.
local PASS_SPEED_FRACTION = 0.5
-- An orbiting ship has orbited after it has turned this far about the centre.
local FULL_TURN           = 2 * math.pi
-- An orbiting ship is on its circle within this fraction of the radius: the wanted velocity is
-- the circle's tangent, and a ship at kilometres per second runs a little wide of it.
local ORBIT_TOLERANCE     = 0.1
-- A direction is taken from a vector at least this long, relative to the radius it is compared with.
local MIN_RELATIVE_LENGTH = 1e-6

---@class SandboxBehaviourSpec
---@field kind "Seek"|"Hold"|"Pursue"|"Follow"|"Orbit"
---@field through number[][]|nil Seek: points { x, y, z } in metres, passed at speed in this order; the ship then stops at the ship's `target`
---@field passRadiusM number|nil Seek: a point is passed within this distance, in metres; longer than the distance covered in one frame
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
---@field phase string|nil Seek: "pass" or "arrive". Hold: "cruise", "switched" or "returning"
---@field points Position[]|nil Seek: the points to pass
---@field pointIndex integer|nil Seek: the point being flown to
---@field passRadius number|nil Seek: game units
---@field onSince number|nil Simulated time the ship has been on station since
---@field turned number Orbit: radians turned about the centre since the ship was last off the circle
---@field previous table|nil Orbit: the ship's last offset from the axis, in the plane of the circle
---@field normal table|nil Orbit: unit normal
---@field radius number|nil Orbit: game units
---@field offset table|nil Follow: the slot, game units
---@field standOff number|nil Pursue: game units

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
    return { kind = spec.kind, spec = spec, engaged = false, maxError = 0, turned = 0, topSpeed = 0 }
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

---Sets `ship.target` to the point a tracking ship is to be at; other ships keep theirs.
---@param ship SandboxShip
function ShipBehaviour.refreshTarget(ship)
    local b = ship.behaviour
    if b and TRACKING[b.kind] then ship.target = toPosition(stationPoint(b, ship)) end
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
function ShipBehaviour.start(nav, ship, ships, obstacles, rules)
    local b, spec = ship.behaviour, ship.behaviour.spec
    b.topSpeed = ConstructComponents.FlightProfile.ProfileOf(ship.entity).maxSpeed
    b.tracked = spec.obstacle and obstacles[spec.obstacle] or spec.ship and ships[spec.ship]
    if b.kind == "Seek" then
        b.points, b.pointIndex, b.phase = {}, 1, "pass"
        for _, point in ipairs(spec.through) do table.insert(b.points, Metres.position(point)) end
        b.passRadius = Metres.length(spec.passRadiusM)
        ship.target = b.points[1]
        nav:setSeek(ship.agent, ship.target, b.passRadius)
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

---Moves a Seek ship on past a point it has passed, and switches a Hold ship to Hold once it is
---flying fast enough. Called after each navigation step.
---@param nav NavWorld
---@param ship SandboxShip
---@param arrived boolean The `NavWorld`'s arrival flag for the ship
---@param simTime number
---@param rules SandboxJudgingRules
---@param metrics ScenarioMetrics
function ShipBehaviour.advance(nav, ship, arrived, simTime, rules, metrics)
    local b = ship.behaviour
    local speed = ship.rb:getSpeed()
    if b.kind == "Seek" and b.phase == "pass" and arrived then
        metrics:seekPassed(ship, b.pointIndex, speed, speed >= PASS_SPEED_FRACTION * b.topSpeed, simTime)
        b.pointIndex = b.pointIndex + 1
        if b.points[b.pointIndex] then
            ship.target = b.points[b.pointIndex]
            nav:setSeek(ship.agent, ship.target, b.passRadius)
        else
            b.phase = "arrive"
            ship.target = Metres.position(ship.spec.target)
            nav:setArrive(ship.agent, ship.target, rules.range)
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
    if distance <= rules.range then b.engaged = true end
    if b.engaged then b.maxError = math.max(b.maxError, distance) end
    local ready = b.kind ~= "Follow" or b.tracked.state ~= "flying"
    return heldOnStation(b, on and ready, simTime)
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
    elseif b.kind == "Seek" then
        return b.phase == "arrive" and isThere(ship, distance, rules)
    end
    -- Hold: the ship must leave its point and come back to it.
    if b.phase == "switched" and distance > rules.range then b.phase = "returning" end
    return b.phase == "returning" and isThere(ship, distance, rules)
end

---A line for the summary: how well a tracking ship kept its station, or nil for the other kinds.
---@param ship SandboxShip
---@return number|nil maxError The largest distance from the station point since it was reached, in game units
function ShipBehaviour.maxStationError(ship)
    local b = ship.behaviour
    return b and TRACKING[b.kind] and b.engaged and b.maxError or nil
end

return ShipBehaviour
