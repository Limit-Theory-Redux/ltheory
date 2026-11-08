--- System planner kernel: a path through a system of orbiting bodies as waypoints. It starts from
--- the straight segment, and where the segment enters a body's safe sphere at the body's predicted
--- position it inserts a waypoint where the tangents from both ends to the sphere meet, then checks
--- the two new legs the same way, down to a bounded depth. Pure numbers in and out (no entities, no
--- globals), so it runs on a worker or in a test state. Not reentrant: it keeps module scratch state.
--- Design: doc/script/system-planner.md.
local OrbitMath = require("Shared.Helpers.OrbitMath")

local sqrt, atan2, acos, sin, cos = math.sqrt, math.atan2, math.acos, math.sin, math.cos
local max, min, abs = math.max, math.min, math.abs

local SystemPlanner = {}

---Result of a plan. The waypoints written are valid for every status except `InvalidInput`,
---`EndpointInsideBody` and `CapacityExceeded` (where they are the ones written before the stop).
SystemPlanner.Status = {
    Ok                 = 0,
    ---A leg still crosses a safe sphere at the depth limit; that leg is left straight.
    DepthExceeded      = 1,
    CapacityExceeded   = 2,
    ---The start or the goal lies inside a body's hull.
    EndpointInsideBody = 3,
    InvalidInput       = 4,
    ---A leg crosses a safe sphere and no tangent exists (an end lies inside it); left straight.
    NoDetour           = 5,
}
local Status = SystemPlanner.Status

-- Body buffer: BODY_STRIDE doubles per body, in the order of these offsets. A parent comes before
-- its children. A body with an orbit radius above zero circles its parent's position (its own
-- position without a parent); any other body stays at its position.
local BODY_STRIDE      = 10
local PARENT           = 0 -- index of the parent body, -1 for none
local POS_X            = 1
local POS_Y            = 2
local POS_Z            = 3
local ORBIT_RADIUS     = 4
local PHASE            = 5
local ANGULAR_SPEED    = 6
local INCLINATION      = 7
local BODY_RADIUS      = 8
local SAFE_RADIUS      = 9
-- Output buffer: OUT_STRIDE doubles per waypoint, the goal last; `t` is the time the ship reaches it.
local OUT_STRIDE       = 4
local OUT_T            = 3

SystemPlanner.BODY_STRIDE = BODY_STRIDE
SystemPlanner.Body = {
    Parent = PARENT, X = POS_X, Y = POS_Y, Z = POS_Z, OrbitRadius = ORBIT_RADIUS, Phase = PHASE,
    AngularSpeed = ANGULAR_SPEED, Inclination = INCLINATION, Radius = BODY_RADIUS, SafeRadius = SAFE_RADIUS,
}
SystemPlanner.OUT_STRIDE = OUT_STRIDE

-- The smallest step of the hit search, as a fraction of the safe radius: it ends a search that
-- grazes a sphere, and can miss only a crossing shallower than this fraction.
local GRAZE_STEP       = 0.01
-- The step that walks through an overlap to find where it ends, as a fraction of the safe radius
-- covered at the fastest relative speed.
local OVERLAP_STEP     = 0.25
-- The detour goes around a sphere this much larger than the safe sphere, so the new legs clear it.
local DETOUR_MARGIN    = 0.1
-- A detour point is at most this many detour radii from the sphere's centre.
local MAX_DETOUR_RADII = 4
-- A body moving across a leg at more than this fraction of the planning speed is passed behind.
local CROSSING_SPEED_FRACTION = 0.1
-- The safe radius of a body that holds an endpoint shrinks to this fraction below the distance.
local ENDPOINT_SHRINK  = 0.01
-- Distances below this make a direction undefined.
local MIN_LENGTH       = 1e-9
local HALF             = 0.5

-- Scratch state of the running plan.
local bodies, bodyCount, speed, maxDepth, out, capacity, written, status, failures
-- Per body: the speed bound of its orbit chain, and the radius of the sphere the hit test uses.
local speedBound, hitRadius = {}, {}

---Where body `i` is `t` seconds from the snapshot.
local function bodyPosition(i, t)
    local base = i * BODY_STRIDE
    local radius = bodies[base + ORBIT_RADIUS]
    if radius <= 0 then
        return bodies[base + POS_X], bodies[base + POS_Y], bodies[base + POS_Z]
    end
    local px, py, pz = bodies[base + POS_X], bodies[base + POS_Y], bodies[base + POS_Z]
    local parent = bodies[base + PARENT]
    if parent >= 0 then px, py, pz = bodyPosition(parent, t) end
    local ox, oy, oz = OrbitMath.offsetAt(radius, bodies[base + PHASE], bodies[base + ANGULAR_SPEED],
        bodies[base + INCLINATION], t)
    return px + ox, py + oy, pz + oz
end

local function isFinite(value)
    return value == value and abs(value) ~= math.huge
end

---Checks the layout and numbers of the input and fills the per-body scratch.
---@return boolean valid
local function prepareBodies()
    for i = 0, bodyCount - 1 do
        local base = i * BODY_STRIDE
        local parent = bodies[base + PARENT]
        local radius, safe = bodies[base + BODY_RADIUS], bodies[base + SAFE_RADIUS]
        if parent ~= parent or parent >= i or parent < -1 or parent ~= math.floor(parent) then return false end
        if not (radius > 0 and safe >= radius) then return false end
        for field = POS_X, INCLINATION do
            if not isFinite(bodies[base + field]) then return false end
        end
        local bound = abs(bodies[base + ORBIT_RADIUS] * bodies[base + ANGULAR_SPEED])
        if parent >= 0 then bound = bound + speedBound[parent + 1] end
        speedBound[i + 1] = bound
        hitRadius[i + 1] = safe
    end
    return true
end

---Shrinks the hit radius of every body whose safe sphere holds the point at time `t`.
---@return boolean ok False when the point is inside a body's hull
local function shrinkAround(x, y, z, t)
    for i = 0, bodyCount - 1 do
        local cx, cy, cz = bodyPosition(i, t)
        local dx, dy, dz = x - cx, y - cy, z - cz
        local d = sqrt(dx * dx + dy * dy + dz * dz)
        local radius = bodies[i * BODY_STRIDE + BODY_RADIUS]
        if d <= radius then return false end
        if d < hitRadius[i + 1] then
            hitRadius[i + 1] = max(radius, d * (1 - ENDPOINT_SHRINK))
        end
    end
    return true
end

---The first body whose sphere the leg enters, and when. The leg runs from `a` along `u` at the
---planning speed from time `t0` to `tEnd`. Each body is searched by stepping in time by the
---smallest time in which the ship and the body could close the gap to the sphere.
---@return integer body Index, -1 for none
---@return number time
local function firstHit(ax, ay, az, ux, uy, uz, t0, tEnd)
    local found, foundTime = -1, tEnd
    for i = 0, bodyCount - 1 do
        local radius = hitRadius[i + 1]
        local closing = speed + speedBound[i + 1]
        local t = t0
        while t <= foundTime do
            local along = speed * (t - t0)
            local cx, cy, cz = bodyPosition(i, t)
            local dx, dy, dz = ax + ux * along - cx, ay + uy * along - cy, az + uz * along - cz
            local d = sqrt(dx * dx + dy * dy + dz * dz)
            if d <= radius then
                found, foundTime = i, t
                break
            end
            t = t + max(d - radius, GRAZE_STEP * radius) / closing
        end
    end
    return found, foundTime
end

---The time in the overlap that is half way between its start and its end.
local function overlapMidTime(i, ax, ay, az, ux, uy, uz, t0, tIn, tEnd)
    local radius = hitRadius[i + 1]
    local step = OVERLAP_STEP * radius / (speed + speedBound[i + 1])
    local last, t = tIn, tIn
    while t <= tEnd do
        local along = speed * (t - t0)
        local cx, cy, cz = bodyPosition(i, t)
        local dx, dy, dz = ax + ux * along - cx, ay + uy * along - cy, az + uz * along - cz
        if sqrt(dx * dx + dy * dy + dz * dz) <= radius then last = t end
        t = t + step
    end
    return (tIn + last) * HALF
end

---A unit vector perpendicular to the unit vector `u`, built from the axis least aligned with it.
local function anyPerpendicular(ux, uy, uz)
    local ax, ay, az = 0, 0, 0
    if abs(ux) <= abs(uy) and abs(ux) <= abs(uz) then ax = 1 elseif abs(uy) <= abs(uz) then ay = 1 else az = 1 end
    local px, py, pz = uy * az - uz * ay, uz * ax - ux * az, ux * ay - uy * ax
    local length = sqrt(px * px + py * py + pz * pz)
    return px / length, py / length, pz / length
end

---The tangent point from the plane point (pu, h) to the circle of radius `r` at the origin that
---lies on the larger-e side.
local function tangentPoint(pu, h, r)
    local d = sqrt(pu * pu + h * h)
    local phi, alpha = atan2(h, pu), acos(min(1, r / d))
    local up, down = phi + alpha, phi - alpha
    local angle = sin(up) >= sin(down) and up or down
    return r * cos(angle), r * sin(angle)
end

---The unit direction, perpendicular to the leg, in which a detour around body `i` bulges. By
---default it points from the centre toward the leg's line (the shorter way round; any
---perpendicular when the line goes through the centre). A body that moves across the leg faster
---than a fraction of the planning speed is passed on the side it is moving away from, so the
---detour point is not where the body is heading. `n` is the vector from the centre to the nearest
---point of the line, and `h` its length.
local function detourSide(i, tm, ux, uy, uz, nx, ny, nz, h)
    local delta = OVERLAP_STEP * hitRadius[i + 1] / (speed + speedBound[i + 1])
    local x1, y1, z1 = bodyPosition(i, tm + delta)
    local x0, y0, z0 = bodyPosition(i, tm - delta)
    local vx, vy, vz = (x1 - x0) / (2 * delta), (y1 - y0) / (2 * delta), (z1 - z0) / (2 * delta)
    local alongSpeed = vx * ux + vy * uy + vz * uz
    local sx, sy, sz = vx - alongSpeed * ux, vy - alongSpeed * uy, vz - alongSpeed * uz
    local sideSpeed = sqrt(sx * sx + sy * sy + sz * sz)
    local crossing = sideSpeed > CROSSING_SPEED_FRACTION * speed

    if h > MIN_LENGTH then
        local ex, ey, ez = nx / h, ny / h, nz / h
        if crossing and sx * ex + sy * ey + sz * ez > 0 then return -ex, -ey, -ez end
        return ex, ey, ez
    end
    if crossing then return -sx / sideSpeed, -sy / sideSpeed, -sz / sideSpeed end
    return anyPerpendicular(ux, uy, uz)
end

---The detour waypoint for a leg that enters body `i`'s sphere: in the plane of the leg and the
---body's centre half way through the overlap, on the side of the leg away from the centre, where
---the tangents from both ends to the detour sphere meet (or a capped point when they don't).
---@return number|nil x
---@return number y
---@return number z
local function detourPoint(i, ax, ay, az, length, ux, uy, uz, t0, tIn, tEnd, flip)
    local tm = overlapMidTime(i, ax, ay, az, ux, uy, uz, t0, tIn, tEnd)
    local cx, cy, cz = bodyPosition(i, tm)
    local wx, wy, wz = cx - ax, cy - ay, cz - az
    local along = wx * ux + wy * uy + wz * uz
    -- n: from the centre to the nearest point of the leg's line
    local nx, ny, nz = ax + ux * along - cx, ay + uy * along - cy, az + uz * along - cz
    local h = sqrt(nx * nx + ny * ny + nz * nz)
    local ex, ey, ez = detourSide(i, tm, ux, uy, uz, nx, ny, nz, h)
    if flip then ex, ey, ez = -ex, -ey, -ez end

    -- The leg's line is `side` from the centre along e (negative when the detour passes on the
    -- centre's side of the line); A and B are at that offset, and at au and bu along the leg.
    local side = (ax - cx) * ex + (ay - cy) * ey + (az - cz) * ez
    local au, bu = -along, length - along
    local dA, dB = sqrt(au * au + side * side), sqrt(bu * bu + side * side)
    local hit = hitRadius[i + 1]
    local nearest = min(dA, dB)
    if nearest <= hit then return nil, 0, 0 end
    local r = min(hit * (1 + DETOUR_MARGIN), hit + (nearest - hit) * HALF)

    local tau, tae = tangentPoint(au, side, r)
    local tbu, tbe = tangentPoint(bu, side, r)
    local det = tau * tbe - tae * tbu
    local pu, pe = 0, MAX_DETOUR_RADII * r
    if abs(det) > MIN_LENGTH then
        local candidateU, candidateE = r * r * (tbe - tae) / det, r * r * (tau - tbu) / det
        if candidateE >= r and sqrt(candidateU * candidateU + candidateE * candidateE) <= MAX_DETOUR_RADII * r then
            pu, pe = candidateU, candidateE
        end
    end
    return cx + ux * pu + ex * pe, cy + uy * pu + ey * pe, cz + uz * pu + ez * pe
end

---Writes a waypoint. Returns false (and sets the status) when the buffer is full.
local function append(x, y, z, t)
    if written >= capacity then
        status = Status.CapacityExceeded
        return false
    end
    local base = written * OUT_STRIDE
    out[base], out[base + 1], out[base + 2], out[base + OUT_T] = x, y, z, t
    written = written + 1
    return true
end

---Records that a leg could not be cleared, keeping the first reason as the status.
local function fail(reason)
    failures = failures + 1
    if status == Status.Ok then status = reason end
end

local planLeg

---Plans the leg a to b through one detour around body `i` and writes its waypoints. `flip`
---passes the body on the other side.
---@return number|nil tEnd The time at b, nil when the plan stopped
local function planDetour(i, ax, ay, az, bx, by, bz, length, ux, uy, uz, t0, tIn, tB, depth, flip)
    local wx, wy, wz = detourPoint(i, ax, ay, az, length, ux, uy, uz, t0, tIn, tB, flip)
    if not wx then
        fail(Status.NoDetour)
        return append(bx, by, bz, tB) and tB or nil
    end
    local tW = planLeg(ax, ay, az, t0, wx, wy, wz, depth + 1)
    if not tW then return nil end
    return planLeg(wx, wy, wz, tW, bx, by, bz, depth + 1)
end

---Plans the leg from a to b, starting at time t0, and writes its waypoints. A detour that fails
---further down (a waypoint inside another body, a leg that cannot be cleared) is redone on the
---other side of the body; if both fail, the first side's result stands.
---@return number|nil tEnd The time at b, nil when the plan stopped
function planLeg(ax, ay, az, t0, bx, by, bz, depth)
    local dx, dy, dz = bx - ax, by - ay, bz - az
    local length = sqrt(dx * dx + dy * dy + dz * dz)
    local tB = t0 + length / speed
    if length <= MIN_LENGTH then
        return append(bx, by, bz, tB) and tB or nil
    end
    local ux, uy, uz = dx / length, dy / length, dz / length
    local body, tIn = firstHit(ax, ay, az, ux, uy, uz, t0, tB)
    if body < 0 or depth >= maxDepth then
        if body >= 0 then fail(Status.DepthExceeded) end
        return append(bx, by, bz, tB) and tB or nil
    end

    local savedWritten, savedFailures, savedStatus = written, failures, status
    local function detour(flip)
        return planDetour(body, ax, ay, az, bx, by, bz, length, ux, uy, uz, t0, tIn, tB, depth, flip)
    end
    local tEnd = detour(false)
    if tEnd == nil or failures == savedFailures then return tEnd end

    written, failures, status = savedWritten, savedFailures, savedStatus
    tEnd = detour(true)
    if tEnd == nil or failures == savedFailures then return tEnd end

    written, failures, status = savedWritten, savedFailures, savedStatus
    return detour(false)
end

---Plans a path from the start to the goal through the bodies.
---@param bodiesBuffer userdata `double[?]`, `bodyCount * BODY_STRIDE` values
---@param count integer Number of bodies
---@param sx number Start x
---@param sy number Start y
---@param sz number Start z
---@param gx number Goal x
---@param gy number Goal y
---@param gz number Goal z
---@param planSpeed number Speed the path is flown at, above zero; sets the times in the output
---@param depthLimit integer Detours nested at most this deep, zero or more
---@param outBuffer userdata `double[?]`, `outCapacity * OUT_STRIDE` values
---@param outCapacity integer Waypoints the output buffer holds
---@return integer status A `Status` value
---@return integer waypointCount Waypoints written, the goal last when the status allows a plan
function SystemPlanner.plan(bodiesBuffer, count, sx, sy, sz, gx, gy, gz, planSpeed, depthLimit, outBuffer, outCapacity)
    bodies, bodyCount, speed, maxDepth, out, capacity = bodiesBuffer, count, planSpeed, depthLimit, outBuffer, outCapacity
    written, status, failures = 0, Status.Ok, 0
    local finiteEndpoints = isFinite(sx) and isFinite(sy) and isFinite(sz) and isFinite(gx) and isFinite(gy)
        and isFinite(gz)
    if not finiteEndpoints or not (planSpeed > 0 and isFinite(planSpeed)) or depthLimit < 0 or outCapacity < 1
        or count < 0 then
        return Status.InvalidInput, 0
    end
    if not prepareBodies() then return Status.InvalidInput, 0 end

    local dx, dy, dz = gx - sx, gy - sy, gz - sz
    local straightTime = sqrt(dx * dx + dy * dy + dz * dz) / planSpeed
    if not (shrinkAround(sx, sy, sz, 0) and shrinkAround(gx, gy, gz, straightTime)) then
        return Status.EndpointInsideBody, 0
    end

    planLeg(sx, sy, sz, 0, gx, gy, gz, 0)
    return status, written
end

return SystemPlanner
