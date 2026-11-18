--- Navigation estimates: what a flight from one point to another would take, without planning it.
--- The straight line, lengthened by a detour factor, is timed with the travel-drive law, slowed
--- where the line crosses a body's drive zone; the risk is the cost of the cost zones the line
--- crosses. Positions are those of the moment the scene was taken: bodies and zones don't move
--- during an estimate. Pure numbers in and out (no entities, no globals), so it runs inline and on a
--- worker.
--- Design: doc/script/nav/navigation.md.
local sqrt, min, max = math.sqrt, math.min, math.max
local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")

---@class NavEstimateScene
---@field bodies number[] `bodyCount * BODY_STRIDE` numbers, laid out by `NavEstimate.Body`
---@field bodyCount integer
---@field zones number[] `zoneCount * ZONE_STRIDE` numbers, laid out by `NavEstimate.Zone`
---@field zoneCount integer

---What flies: the ship's top speed and drag, the drive's open-space multiplier and fall rate (nil
---without a drive), and how much longer than the straight line a real flight is.
---@class NavEstimateShip
---@field maxSpeed number Game units per second, without the drive
---@field linearDrag number Per second
---@field openCap number|nil The drive multiplier in open space; nil for a ship with no drive
---@field rampDownSpeed number|nil The drive multiplier's fall rate, per second
---@field detourFactor number Path length over straight distance, 1 or more

---@class NavEstimateResult
---@field reachable boolean False when the start or the goal is inside a body's hull
---@field time number|nil Seconds until the ship is within the arrival radius of the goal
---@field distance gameunit|nil The path's length
---@field risk number|nil Cost zone cost times the length crossed, summed (0 with no zones)

local BODY_X, BODY_Y, BODY_Z, BODY_HULL, BODY_ZONE_RADIUS, BODY_ZONE_CAP, BODY_STRIDE = 0, 1, 2, 3, 4, 5, 6
local ZONE_X, ZONE_Y, ZONE_Z, ZONE_RADIUS, ZONE_COST, ZONE_STRIDE = 0, 1, 2, 3, 4, 5
local GOAL_X, GOAL_Y, GOAL_Z, GOAL_ARRIVAL, GOAL_DRIVE_MIN, GOAL_STRIDE = 0, 1, 2, 3, 4, 5
local RESULT_REACHABLE, RESULT_TIME, RESULT_DISTANCE, RESULT_RISK, RESULT_STRIDE = 0, 1, 2, 3, 4

-- A stretch of the line inside one or more drive zones' influence is integrated at this many steps.
local ZONE_SAMPLES = 32

local NavEstimate = {
    -- The arrival radius of a goal that doesn't exist (its target is gone): it is not reachable.
    NO_GOAL = -1,
    BODY_STRIDE = BODY_STRIDE,
    -- A body: its centre, hull radius, drive zone radius (0 for none) and the drive multiplier at its centre.
    Body = { X = BODY_X, Y = BODY_Y, Z = BODY_Z, Hull = BODY_HULL, ZoneRadius = BODY_ZONE_RADIUS, ZoneCap = BODY_ZONE_CAP },
    ZONE_STRIDE = ZONE_STRIDE,
    -- A cost zone: its centre, radius and cost per unit of length crossed.
    Zone = { X = ZONE_X, Y = ZONE_Y, Z = ZONE_Z, Radius = ZONE_RADIUS, Cost = ZONE_COST },
    GOAL_STRIDE = GOAL_STRIDE,
    -- A goal: the point the ship stops at, how close counts as arrived, and the shortest flight the drive is used on.
    Goal = { X = GOAL_X, Y = GOAL_Y, Z = GOAL_Z, ArrivalRadius = GOAL_ARRIVAL, DriveMinLength = GOAL_DRIVE_MIN },
    RESULT_STRIDE = RESULT_STRIDE,
    Result = { Reachable = RESULT_REACHABLE, Time = RESULT_TIME, Distance = RESULT_DISTANCE, Risk = RESULT_RISK },
}

---Whether a point is inside a body's hull.
---@param scene NavEstimateScene
---@return boolean
local function insideHull(scene, x, y, z)
    local bodies = scene.bodies
    for i = 0, scene.bodyCount - 1 do
        local base = i * BODY_STRIDE + 1
        local dx, dy, dz = x - bodies[base + BODY_X], y - bodies[base + BODY_Y], z - bodies[base + BODY_Z]
        local hull = bodies[base + BODY_HULL]
        if dx * dx + dy * dy + dz * dz < hull * hull then return true end
    end
    return false
end

---The part of the segment from (ax, ay, az) along the unit vector u that is inside a sphere.
---@return number|nil from Distance along the segment; nil when it misses the sphere
---@return number|nil to
local function chord(ax, ay, az, ux, uy, uz, length, cx, cy, cz, radius)
    local wx, wy, wz = cx - ax, cy - ay, cz - az
    local along = wx * ux + wy * uy + wz * uz
    local reach2 = radius * radius - (wx * wx + wy * wy + wz * wz - along * along)
    if reach2 <= 0 then return nil end
    local half = sqrt(reach2)
    local from, to = max(0, along - half), min(length, along + half)
    if to <= from then return nil end
    return from, to
end

---The cost zones' risk: each zone's cost times the length of the line inside it.
local function riskAlong(scene, ax, ay, az, ux, uy, uz, length)
    local zones, risk = scene.zones, 0
    for i = 0, scene.zoneCount - 1 do
        local base = i * ZONE_STRIDE + 1
        local from, to = chord(ax, ay, az, ux, uy, uz, length, zones[base + ZONE_X], zones[base + ZONE_Y],
            zones[base + ZONE_Z], zones[base + ZONE_RADIUS])
        if from then risk = risk + zones[base + ZONE_COST] * (to - from) end
    end
    return risk
end

---The stretches of the line within the influence of a body's drive zone, sorted and merged.
---@return number[][] intervals `{ from, to }` pairs
local function zoneIntervals(scene, ax, ay, az, ux, uy, uz, length)
    local bodies, found = scene.bodies, {}
    for i = 0, scene.bodyCount - 1 do
        local base = i * BODY_STRIDE + 1
        local radius = bodies[base + BODY_ZONE_RADIUS]
        if radius > 0 then
            local from, to = chord(ax, ay, az, ux, uy, uz, length, bodies[base + BODY_X], bodies[base + BODY_Y],
                bodies[base + BODY_Z], radius * TravelDriveMath.INFLUENCE_RADIUS_FACTOR)
            if from then found[#found + 1] = { from, to } end
        end
    end
    table.sort(found, function(a, b) return a[1] < b[1] end)
    local merged = {}
    for _, interval in ipairs(found) do
        local last = merged[#merged]
        if last and interval[1] <= last[2] then
            last[2] = max(last[2], interval[2])
        else
            merged[#merged + 1] = interval
        end
    end
    return merged
end

---The lowest drive multiplier the bodies' zones allow at a point.
local function zoneMultiplier(scene, x, y, z, openCap)
    local bodies, lowest = scene.bodies, openCap
    for i = 0, scene.bodyCount - 1 do
        local base = i * BODY_STRIDE + 1
        local radius = bodies[base + BODY_ZONE_RADIUS]
        if radius > 0 then
            local dx, dy, dz = x - bodies[base + BODY_X], y - bodies[base + BODY_Y], z - bodies[base + BODY_Z]
            lowest = min(lowest, TravelDriveMath.zoneCap(openCap, bodies[base + BODY_ZONE_CAP], radius,
                sqrt(dx * dx + dy * dy + dz * dz)))
        end
    end
    return lowest
end

---The seconds a flight under the drive loses in the drive zones the line crosses: at each step the
---ship goes at the zone's multiplier instead of the one the braking law gives in open space, where
---the drive is on (the law wants the hand-over multiplier or more).
local function zoneDelay(scene, ship, ax, ay, az, ux, uy, uz, length)
    local braking = TravelDriveMath.brakingTime(ship.rampDownSpeed, ship.linearDrag)
    local delay = 0
    for _, interval in ipairs(zoneIntervals(scene, ax, ay, az, ux, uy, uz, length)) do
        local step = (interval[2] - interval[1]) / ZONE_SAMPLES
        for k = 0, ZONE_SAMPLES - 1 do
            local s = interval[1] + (k + 0.5) * step
            local free = TravelDriveMath.cappedMultiplier(
                TravelDriveMath.brakingMultiplier((length - s) * ship.detourFactor, ship.maxSpeed, braking), ship.openCap)
            if free >= TravelDriveMath.HANDOVER_MULT then
                local held = TravelDriveMath.cappedMultiplier(free,
                    zoneMultiplier(scene, ax + ux * s, ay + uy * s, az + uz * s, ship.openCap))
                delay = delay + step * ship.detourFactor * (1 / held - 1 / free) / ship.maxSpeed
            end
        end
    end
    return delay
end

---Estimates one flight.
---@param scene NavEstimateScene
---@param ship NavEstimateShip
---@param start number[] { x, y, z }
---@param goals number[] `NavEstimate.Goal` layout, `GOAL_STRIDE` numbers per goal, 1-based
---@param index integer 0-based goal index
---@return NavEstimateResult
local function estimateOne(scene, ship, start, goals, index)
    local base = index * GOAL_STRIDE + 1
    local gx, gy, gz = goals[base + GOAL_X], goals[base + GOAL_Y], goals[base + GOAL_Z]
    local ax, ay, az = start[1], start[2], start[3]
    if goals[base + GOAL_ARRIVAL] == NavEstimate.NO_GOAL or insideHull(scene, ax, ay, az)
        or insideHull(scene, gx, gy, gz) then
        return { reachable = false }
    end

    local dx, dy, dz = gx - ax, gy - ay, gz - az
    local length = sqrt(dx * dx + dy * dy + dz * dz)
    local distance = length * ship.detourFactor
    if length == 0 then return { reachable = true, time = 0, distance = 0, risk = 0 } end
    local ux, uy, uz = dx / length, dy / length, dz / length

    local driveUsed = ship.openCap ~= nil and distance > goals[base + GOAL_DRIVE_MIN]
    local time = TravelDriveMath.travelTime(distance, goals[base + GOAL_ARRIVAL], ship.maxSpeed,
        ship.rampDownSpeed or 1, ship.linearDrag, ship.openCap or 1, driveUsed)
    if driveUsed then time = time + zoneDelay(scene, ship, ax, ay, az, ux, uy, uz, length) end
    return { reachable = true, time = time, distance = distance, risk = riskAlong(scene, ax, ay, az, ux, uy, uz, length) }
end

---Estimates a flight from `start` to each goal.
---@param scene NavEstimateScene
---@param ship NavEstimateShip
---@param start number[] { x, y, z }
---@param goals number[] `goalCount * GOAL_STRIDE` numbers
---@param goalCount integer
---@return NavEstimateResult[] results In the goals' order
function NavEstimate.many(scene, ship, start, goals, goalCount)
    local results = {}
    for i = 0, goalCount - 1 do results[i + 1] = estimateOne(scene, ship, start, goals, i) end
    return results
end

return NavEstimate
