--- System planner kernel: a path through a system of orbiting bodies as waypoints. It starts from
--- the straight segment, and where the segment enters a body's safe sphere at the body's predicted
--- position it inserts a waypoint where the tangents from both ends to the sphere meet, then checks
--- the two new legs the same way, down to a bounded depth. Each corner is then rounded into an arc
--- with a radius and a speed the ship can hold it at. A waypoint is a body index and an offset
--- from that body (the system frame for none). With a travel-drive block, each leg is timed under
--- the drive where that is sooner, and the drive ends before every corner. Pure numbers in and out
--- (no entities, no globals), so it runs on a worker or in a test state. Not reentrant: it keeps
--- module scratch state.
--- Design: doc/script/system-planner.md.
local OrbitMath       = require("Shared.Helpers.OrbitMath")
local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")

local sqrt, atan2, acos, sin, cos, tan, exp = math.sqrt, math.atan2, math.acos, math.sin, math.cos, math.tan, math.exp
local max, min, abs = math.max, math.min, math.abs

local SystemPlanner = {}

---Result of a plan. The waypoints written are valid for every status except `InvalidInput`,
---`EndpointInsideBody`, `GoalOutrun` and `CapacityExceeded` (where they are the ones written
---before the stop).
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
    ---The goal's frame body can move as fast as the ship, so the ship may never catch the goal.
    GoalOutrun         = 6,
    ---The arrival time at a goal on a moving body did not settle within the iteration limit; the
    ---last iteration's plan is kept, and its times may be off.
    GoalNotSettled     = 7,
    ---A leg needed more drive timeline steps than the limit; it is planned without the drive.
    DriveNotTimed      = 8,
}
local Status = SystemPlanner.Status

-- Body buffer: BODY_STRIDE doubles per body, in the order of these offsets. A parent comes before
-- its children. A body with an orbit radius above zero circles its parent's position (its own
-- position without a parent); any other body stays at its position. A body with a zone radius
-- above zero caps the travel drive near it (`TravelDriveMath.zoneCap`): its zone cap is the
-- multiplier at its centre.
local BODY_STRIDE      = 12
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
local ZONE_RADIUS      = 10
local ZONE_CAP         = 11
-- Output buffer: OUT_STRIDE doubles per waypoint, the goal last. A waypoint is the index of its
-- frame body (SYSTEM_FRAME for the system) and the offset from that body's centre, in world axes,
-- and `t`, the time the ship reaches it. The frame's position at `t` plus the offset is the point.
-- Radius and speed describe the corner there: the radius of the arc that rounds it (zero for a
-- sharp corner) and the speed the ship is to pass at. `drive` is 1 when the leg that ends at the
-- waypoint is flown under the travel drive, which is released before the waypoint.
local OUT_FRAME        = 0
local OUT_X            = 1
local OUT_Y            = 2
local OUT_Z            = 3
local OUT_T            = 4
local OUT_RADIUS       = 5
local OUT_SPEED        = 6
local OUT_DRIVE        = 7
local OUT_STRIDE       = 8
local SYSTEM_FRAME     = -1
-- Drive buffer: DRIVE_STRIDE doubles, in the order of these offsets. The multiplier in open
-- space, the seconds the drive charges at normal speed, and the rates at which its multiplier
-- rises (per second, times the multiplier above 1) and falls (the rate of an exponential).
local DRIVE_OPEN_CAP   = 0
local DRIVE_CHARGE     = 1
local DRIVE_RAMP_UP    = 2
local DRIVE_RAMP_DOWN  = 3
local DRIVE_STRIDE     = 4

SystemPlanner.BODY_STRIDE = BODY_STRIDE
SystemPlanner.Body = {
    Parent = PARENT, X = POS_X, Y = POS_Y, Z = POS_Z, OrbitRadius = ORBIT_RADIUS, Phase = PHASE,
    AngularSpeed = ANGULAR_SPEED, Inclination = INCLINATION, Radius = BODY_RADIUS, SafeRadius = SAFE_RADIUS,
    ZoneRadius = ZONE_RADIUS, ZoneCap = ZONE_CAP,
}
SystemPlanner.OUT_STRIDE = OUT_STRIDE
SystemPlanner.Out = {
    Frame = OUT_FRAME, X = OUT_X, Y = OUT_Y, Z = OUT_Z, T = OUT_T, Radius = OUT_RADIUS, Speed = OUT_SPEED,
    Drive = OUT_DRIVE,
}
SystemPlanner.DRIVE_STRIDE = DRIVE_STRIDE
SystemPlanner.Drive = {
    OpenCap = DRIVE_OPEN_CAP, Charge = DRIVE_CHARGE, RampUp = DRIVE_RAMP_UP, RampDown = DRIVE_RAMP_DOWN,
}
SystemPlanner.SYSTEM_FRAME = SYSTEM_FRAME

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
-- A goal on a moving body is re-planned for the time the last plan arrives at it, until that time
-- changes by less than the tolerance (seconds) or the iteration limit is reached. Each pass
-- shrinks the error by the goal's speed over the planning speed. The tolerance is above the
-- jitter that the stepped overlap search puts on a detour point (about 1e-5 s).
local GOAL_TIME_TOLERANCE = 1e-4
local MAX_GOAL_ITERATIONS = 50

-- The share of the forward thrust a ship spends on holding a turn: the turn model's counterpart of
-- `ORBIT_THRUST_SHARE` in `nav`. The rest is left for the velocity loop's lag and for avoidance.
local TURN_THRUST_SHARE = 0.6
-- A corner's arc is no larger than the one at which the turn speed reaches this fraction of its
-- limit (the speed at which drag alone uses the turn's thrust share): a larger arc buys little
-- speed and costs path length.
local TURN_SPEED_KNEE   = 0.9
-- The two tangent lengths of an arc are at most this share of the legs they cut, so the arcs of
-- neighbouring corners never overlap.
local ARC_LEG_SHARE     = 0.5
-- An arc is checked against the bodies as this many chords.
local ARC_CHORDS        = 8
-- An arc that hits a body is made half as large, up to this many times, before the corner is left
-- sharp.
local MAX_ARC_HALVINGS  = 4
-- A corner that turns by less than this (radians) is not rounded and is passed at the planning speed.
local MIN_ARC_ANGLE     = 0.02

SystemPlanner.TURN_THRUST_SHARE = TURN_THRUST_SHARE

-- A drive leg is timed in steps. Each step covers at most this share of the distance to the
-- nearest feature that changes the drive's cap: a zone's edge or body, or the point where the
-- braking law starts to bind.
local DRIVE_FEATURE_STEP = 0.05
-- The distance to a zone's edge counts as at least this share of the zone's influence radius, so
-- a ship on the edge still takes steps.
local DRIVE_FEATURE_FLOOR = 0.01
-- While the drive's multiplier or the ship's speed is off its target by more than this share, a
-- step is at most `DRIVE_TRANSIENT_STEP` of the shortest time constant that governs them.
local DRIVE_TRANSIENT_TOLERANCE = 1e-3
local DRIVE_TRANSIENT_STEP = 0.1
-- The shortest step (seconds), which ends a search that would creep, and the most steps of one
-- leg, past which the leg is flown without the drive.
local DRIVE_MIN_STEP_S = 1e-6
local MAX_DRIVE_STEPS = 20000

-- Scratch state of the running plan.
local bodies, bodyCount, speed, maxDepth, out, capacity, written, status, failures, goalFrame
-- The ship's forward acceleration and drag, its top speed (infinite without drag) and the arc
-- radius at the turn-speed knee (infinite without drag).
local shipAccel, shipDrag, shipTopSpeed, kneeRadius
-- The travel drive: whether it is planned, the multiplier in open space, the charge time, the
-- rise and fall rates, the shortest time constant of the drive and the ship's speed, the braking
-- law's time constant and the distance from a leg's end at which the drive is released.
local driveOn, driveOpenCap, driveCharge, driveRampUp, driveRampDown = false, 1, 0, 1, 1
local driveShortestLag, driveBrakingTime, driveRelease = 1, 1, 0
-- Per body: the speed bound of its orbit chain, and the radius of the sphere the hit test uses.
local speedBound, hitRadius = {}, {}
-- The timeline of a leg: its samples (time, distance along the leg) and, per interval between two
-- samples, the greatest speed in it. A leg's profile is kept per recursion depth, so a retry on
-- the other side of a body finds its parent's intact; chords of an arc use their own.
local legProfiles, chordProfile = {}, { t = {}, s = {}, vmax = {}, count = 0, drive = false }

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

---Whether a body's zone fields are usable: no zone (radius zero), or a finite radius and a cap of
---at least 1 (the drive multiplier is never below 1).
local function zoneValid(base)
    local zoneRadius, zoneCap = bodies[base + ZONE_RADIUS], bodies[base + ZONE_CAP]
    if not (isFinite(zoneRadius) and zoneRadius >= 0) then return false end
    return zoneRadius == 0 or (isFinite(zoneCap) and zoneCap >= 1)
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
        if not zoneValid(base) then return false end
        local bound = abs(bodies[base + ORBIT_RADIUS] * bodies[base + ANGULAR_SPEED])
        if parent >= 0 then bound = bound + speedBound[parent + 1] end
        speedBound[i + 1] = bound
    end
    return true
end

---Restores every body's hit radius to its safe radius, undoing the endpoint shrinks of a pass.
local function resetHitRadii()
    for i = 0, bodyCount - 1 do
        hitRadius[i + 1] = bodies[i * BODY_STRIDE + SAFE_RADIUS]
    end
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

---The profile of the leg at recursion depth `depth`.
local function legProfile(depth)
    local profile = legProfiles[depth + 1]
    if not profile then
        profile = { t = {}, s = {}, vmax = {}, count = 0, drive = false }
        legProfiles[depth + 1] = profile
    end
    return profile
end

---Fills `profile` with a leg of `length` flown at the planning speed from time `t0`.
local function flatProfile(profile, t0, length)
    profile.t[1], profile.s[1], profile.vmax[1] = t0, 0, speed
    profile.t[2], profile.s[2] = t0 + length / speed, length
    profile.count, profile.drive = 2, false
end

---What the zones do to the drive at the point (x, y, z) at time `t`, for a ship moving at `v`: the
---multiplier they allow (the lowest of any zone's, at most the open-space cap) and the longest
---step in seconds before the cap could have changed much. A step covers at most
---`DRIVE_FEATURE_STEP` of the distance to a zone's edge, counted relative to the zone's body and
---at its speed bound.
---@return number limit
---@return number step
local function zoneState(x, y, z, t, v)
    local limit, step = driveOpenCap, math.huge
    for i = 0, bodyCount - 1 do
        local base = i * BODY_STRIDE
        local zoneRadius = bodies[base + ZONE_RADIUS]
        if zoneRadius > 0 then
            local cx, cy, cz = bodyPosition(i, t)
            local dx, dy, dz = x - cx, y - cy, z - cz
            local d = sqrt(dx * dx + dy * dy + dz * dz)
            limit = min(limit, TravelDriveMath.zoneCap(driveOpenCap, bodies[base + ZONE_CAP], zoneRadius, d))
            local influence = zoneRadius * TravelDriveMath.INFLUENCE_RADIUS_FACTOR
            local gap = max(abs(d - influence), DRIVE_FEATURE_FLOOR * influence)
            step = min(step, DRIVE_FEATURE_STEP * gap / (v + speedBound[i + 1]))
        end
    end
    return limit, step
end

---One step of the drive's multiplier `m` and the ship's speed `v` over `dt` seconds toward a
---target multiplier: it rises by `(target - 1) · rampUp` per second and falls toward the target
---exponentially; the speed follows the drive's top speed `speed · m` with the ship's drag.
---@return number m
---@return number v
local function driveStep(m, v, target, dt)
    local mNew
    if m < target then
        mNew = min(target, m + (target - 1) * driveRampUp * dt)
    else
        mNew = target + (m - target) * exp(-driveRampDown * dt)
    end
    local vTarget = speed * mNew
    return mNew, vTarget + (v - vTarget) * exp(-shipDrag * dt)
end

---Fills `profile` with the leg of `length` from `a` along `u` as it is flown under the drive from
---time `t0`, and returns whether that worked. The ship flies at the planning speed while the
---drive charges, then the drive follows the lowest of the zones' cap at the ship's point and
---time and the braking law's multiplier for the distance left, and it is released `driveRelease`
---before the leg's end, from where the leg is flown at the planning speed. The timeline is a
---chain of samples, each step bounded by the zones' features, by the share of the distance left
---and, while the drive or the speed is off its target, by a share of the shortest time constant.
---False for a leg too short to charge and release, or one that needs more than `MAX_DRIVE_STEPS`.
local function driveProfile(profile, t0, length, ax, ay, az, ux, uy, uz)
    local sRelease = length - driveRelease
    local t, s, m, v = t0 + driveCharge, speed * driveCharge, 1, speed
    if s >= sRelease then return false end
    local times, along, vmax = profile.t, profile.s, profile.vmax
    times[1], along[1] = t0, 0
    local count = 1
    if driveCharge > 0 then
        times[2], along[2], vmax[1], count = t, s, speed, 2
    end
    for _ = 1, MAX_DRIVE_STEPS do
        local limit, dt = zoneState(ax + ux * s, ay + uy * s, az + uz * s, t, v)
        local distLeft = length - s
        local wanted = TravelDriveMath.brakingMultiplier(distLeft, speed, driveBrakingTime)
        local target = max(1, min(limit, wanted))
        dt = min(dt, DRIVE_FEATURE_STEP * distLeft / v)
        local offTarget = abs(m - target) > DRIVE_TRANSIENT_TOLERANCE * target
        local lagging = abs(v - speed * m) > DRIVE_TRANSIENT_TOLERANCE * v
        if offTarget or lagging then
            dt = min(dt, DRIVE_TRANSIENT_STEP * driveShortestLag)
        end
        dt = max(dt, DRIVE_MIN_STEP_S)
        local mNew, vNew = driveStep(m, v, target, dt)
        local sNew = s + HALF * (v + vNew) * dt
        local released = sNew >= sRelease
        if released then
            dt = dt * (sRelease - s) / (sNew - s)
            sNew = sRelease
        end
        count = count + 1
        times[count], along[count], vmax[count - 1] = t + dt, sNew, max(v, vNew)
        t, s, m, v = t + dt, sNew, mNew, vNew
        if released then
            times[count + 1], along[count + 1], vmax[count] = t + (length - s) / speed, length, speed
            profile.count, profile.drive = count + 1, true
            return true
        end
    end
    return false
end

---Fills `profile` with the leg of `length` from `a` along `u` from time `t0`: under the drive when
---that gets to the end sooner, else at the planning speed.
local function buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz)
    if driveOn then
        local timed = driveProfile(profile, t0, length, ax, ay, az, ux, uy, uz)
        if timed and profile.t[profile.count] < t0 + length / speed then return end
        if not timed and length > driveRelease + speed * driveCharge and status == Status.Ok then
            status = Status.DriveNotTimed
        end
    end
    flatProfile(profile, t0, length)
end

---The index of the interval of the profile that holds time `t`, searching on from interval `k`.
---An interval ends where the next begins, so a time on a boundary belongs to the later one.
local function locate(profile, t, k)
    local times, last = profile.t, profile.count - 1
    while k < last and t >= times[k + 1] do k = k + 1 end
    return k
end

---How far along the leg the ship is at time `t`, which lies in interval `k`.
local function distanceAt(profile, t, k)
    local t1, t2 = profile.t[k], profile.t[k + 1]
    local s1, s2 = profile.s[k], profile.s[k + 1]
    if t >= t2 or t2 <= t1 then return s2 end
    return s1 + (s2 - s1) * (t - t1) / (t2 - t1)
end

---The distance from the ship, `t` seconds in (interval `k` of the profile), to the centre of body `i`.
local function gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t)
    local along = distanceAt(profile, t, k)
    local cx, cy, cz = bodyPosition(i, t)
    local dx, dy, dz = ax + ux * along - cx, ay + uy * along - cy, az + uz * along - cz
    return sqrt(dx * dx + dy * dy + dz * dz)
end

---The next time of a step of `step` seconds from `t` in interval `k`, held to the interval's end:
---the speed bound of an interval holds only inside it.
local function stepTo(profile, k, t, step)
    local tNext = t + step
    if k < profile.count - 1 and tNext > profile.t[k + 1] then return profile.t[k + 1] end
    return tNext
end

---The first time up to `tLimit`, from `t0`, that the leg is inside body `i`'s hit sphere. The leg
---runs from `a` along `u`. The search steps in time by the smallest time in which the ship
---(at the greatest speed of its interval) and the body could close the gap to the sphere.
---@return number|nil time Nil when the leg stays outside
local function bodyHit(profile, i, ax, ay, az, ux, uy, uz, t0, tLimit)
    local radius, bound = hitRadius[i + 1], speedBound[i + 1]
    local t, k = t0, 1
    while t <= tLimit do
        k = locate(profile, t, k)
        local d = gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t)
        if d <= radius then return t end
        t = stepTo(profile, k, t, max(d - radius, GRAZE_STEP * radius) / (profile.vmax[k] + bound))
    end
    return nil
end

---The first body whose sphere the leg enters, and when. The leg runs from `a` along `u` through
---the profile's times, from `t0` to `tEnd`.
---@return integer body Index, -1 for none
---@return number time
local function firstHit(profile, ax, ay, az, ux, uy, uz, t0, tEnd)
    local found, foundTime = -1, tEnd
    for i = 0, bodyCount - 1 do
        local hit = bodyHit(profile, i, ax, ay, az, ux, uy, uz, t0, foundTime)
        if hit then found, foundTime = i, hit end
    end
    return found, foundTime
end

---The time in the overlap that is half way between its start and its end. The overlap ends at
---the last time the leg is inside the sphere: the walk steps through the sphere by a share of its
---radius, and after each exit looks ahead for a re-entry.
local function overlapMidTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    local radius, bound = hitRadius[i + 1], speedBound[i + 1]
    local last, t, k = tIn, tIn, 1
    while t <= tEnd do
        k = locate(profile, t, k)
        if gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t) <= radius then
            last = t
            t = stepTo(profile, k, t, OVERLAP_STEP * radius / (profile.vmax[k] + bound))
        else
            local reentry = bodyHit(profile, i, ax, ay, az, ux, uy, uz, t, tEnd)
            if not reentry then break end
            t = reentry
        end
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
local function detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tEnd, flip)
    local tm = overlapMidTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
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

---Writes the world point (x, y, z) reached at time `t` as an offset from the frame body's position
---at `t` (the point itself for the system frame), reached by a leg flown under the drive or not.
---Returns false (and sets the status) when the buffer is full.
local function append(x, y, z, t, frame, drive)
    if written >= capacity then
        status = Status.CapacityExceeded
        return false
    end
    if frame ~= SYSTEM_FRAME then
        local cx, cy, cz = bodyPosition(frame, t)
        x, y, z = x - cx, y - cy, z - cz
    end
    local base = written * OUT_STRIDE
    out[base + OUT_FRAME], out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z], out[base + OUT_T] =
        frame, x, y, z, t
    out[base + OUT_RADIUS], out[base + OUT_SPEED], out[base + OUT_DRIVE] = 0, 0, drive and 1 or 0
    written = written + 1
    return true
end

---Records that a leg could not be cleared, keeping the first reason as the status.
local function fail(reason)
    failures = failures + 1
    if status == Status.Ok then status = reason end
end

local planLeg

---Plans the leg a to b, whose timeline is `profile`, through one detour around body `i` and writes
---its waypoints; the detour waypoint is in body `i`'s frame, and b in `bFrame`. `flip` passes the
---body on the other side.
---@return number|nil tEnd The time at b, nil when the plan stopped
local function planDetour(profile, i, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0, tIn, tB, depth, flip)
    local wx, wy, wz = detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tB, flip)
    if not wx then
        fail(Status.NoDetour)
        return append(bx, by, bz, tB, bFrame, profile.drive) and tB or nil
    end
    local tW = planLeg(ax, ay, az, t0, wx, wy, wz, i, depth + 1)
    if not tW then return nil end
    return planLeg(wx, wy, wz, tW, bx, by, bz, bFrame, depth + 1)
end

---Plans the leg from a to b (a point in frame `bFrame`), starting at time t0, and writes its
---waypoints. The leg's timeline is the drive's where that is sooner. A detour that fails
---further down (a waypoint inside another body, a leg that cannot be cleared) is redone on the
---other side of the body; if both fail, the first side's result stands.
---@return number|nil tEnd The time at b, nil when the plan stopped
function planLeg(ax, ay, az, t0, bx, by, bz, bFrame, depth)
    local dx, dy, dz = bx - ax, by - ay, bz - az
    local length = sqrt(dx * dx + dy * dy + dz * dz)
    if length <= MIN_LENGTH then
        local tB = t0 + length / speed
        return append(bx, by, bz, tB, bFrame, false) and tB or nil
    end
    local ux, uy, uz = dx / length, dy / length, dz / length
    -- The leg's profile lives at its depth: the legs of a detour use the next depth, so a retry on
    -- the other side finds this one intact.
    local profile = legProfile(depth)
    buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz)
    local tB = profile.t[profile.count]
    local body, tIn = firstHit(profile, ax, ay, az, ux, uy, uz, t0, tB)
    if body < 0 or depth >= maxDepth then
        if body >= 0 then fail(Status.DepthExceeded) end
        return append(bx, by, bz, tB, bFrame, profile.drive) and tB or nil
    end

    local savedWritten, savedFailures, savedStatus = written, failures, status
    local function detour(flip)
        return planDetour(profile, body, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0, tIn, tB, depth, flip)
    end
    local tEnd = detour(false)
    if tEnd == nil or failures == savedFailures then return tEnd end

    written, failures, status = savedWritten, savedFailures, savedStatus
    tEnd = detour(true)
    if tEnd == nil or failures == savedFailures then return tEnd end

    written, failures, status = savedWritten, savedFailures, savedStatus
    return detour(false)
end

---The speed at which `TURN_THRUST_SHARE` of the forward acceleration `accel` holds a circle of
---`radius` against the linear drag `drag` and the centripetal demand: the positive root of
---`s⁴/r² + k²s² = (share·a)²`, at most `topSpeed`. Zero for a circle without radius or a ship
---without thrust. It is the formula of `nav::circle_speed`, which a test holds it to.
---@param accel number Forward acceleration, game units per second squared
---@param drag number Linear drag rate, per second, zero or more
---@param radius number Game units
---@param topSpeed number Game units per second, infinite for none
---@return number speed Game units per second
function SystemPlanner.turnSpeed(accel, drag, radius, topSpeed)
    local thrust = TURN_THRUST_SHARE * accel
    if radius <= 0 or thrust <= 0 then return 0 end
    local dragSq = drag * drag
    local speedSq = HALF * radius * radius
        * (-dragSq + sqrt(dragSq * dragSq + 4 * thrust * thrust / (radius * radius)))
    return min(sqrt(speedSq), topSpeed)
end

---The turn at a corner: its angle, the lengths of the two legs and their unit directions. Nil
---for a leg without length.
local function cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    local ax, ay, az, bx, by, bz = cx - px, cy - py, cz - pz, nx - cx, ny - cy, nz - cz
    local la, lb = sqrt(ax * ax + ay * ay + az * az), sqrt(bx * bx + by * by + bz * bz)
    if la <= MIN_LENGTH or lb <= MIN_LENGTH then return nil end
    local u1x, u1y, u1z, u2x, u2y, u2z = ax / la, ay / la, az / la, bx / lb, by / lb, bz / lb
    local turn = acos(max(-1, min(1, u1x * u2x + u1y * u2y + u1z * u2z)))
    return turn, la, lb, u1x, u1y, u1z, u2x, u2y, u2z
end

---The arc of `radius` that rounds the corner `c` between the points before it (`p`) and after it
---(`n`): it is tangent to both legs, so its entry and exit are `radius · tan(turn / 2)` from the
---corner along them. The one place this geometry is written: the planner checks the arc with it,
---and a ship that flies the plan builds the same arc from the corner's points as they are now.
---Nil for a corner that has no arc: a leg without length, a turn below `MIN_ARC_ANGLE`, or a
---reversal.
---@return number|nil entryX
---@return number entryY
---@return number entryZ
---@return number exitX
---@return number exitY
---@return number exitZ
---@return number centreX
---@return number centreY
---@return number centreZ
---@return number normalX Unit normal: the ship turns anticlockwise about it
---@return number normalY
---@return number normalZ
---@return number turn Radians swept from the entry to the exit
function SystemPlanner.cornerArc(px, py, pz, cx, cy, cz, nx, ny, nz, radius)
    local turn, _, _, u1x, u1y, u1z, u2x, u2y, u2z = cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    if not turn or turn < MIN_ARC_ANGLE then return nil end
    local kx, ky, kz = u1y * u2z - u1z * u2y, u1z * u2x - u1x * u2z, u1x * u2y - u1y * u2x
    local kLength = sqrt(kx * kx + ky * ky + kz * kz)
    if kLength <= MIN_LENGTH then return nil end
    local tangent = radius * tan(turn * HALF)
    local ex, ey, ez = cx - u1x * tangent, cy - u1y * tangent, cz - u1z * tangent
    -- The centre is across the first leg, toward the second one.
    local along = u1x * u2x + u1y * u2y + u1z * u2z
    local ix, iy, iz = u2x - u1x * along, u2y - u1y * along, u2z - u1z * along
    local iLength = sqrt(ix * ix + iy * iy + iz * iz)
    return ex, ey, ez,
        cx + u2x * tangent, cy + u2y * tangent, cz + u2z * tangent,
        ex + ix / iLength * radius, ey + iy / iLength * radius, ez + iz / iLength * radius,
        kx / kLength, ky / kLength, kz / kLength, turn
end

---The point of an arc `angle` radians (zero to the arc's turn) from its entry, for the centre,
---the entry and the unit normal `cornerArc` returns.
function SystemPlanner.pointOnArc(ox, oy, oz, ex, ey, ez, kx, ky, kz, angle)
    local vx, vy, vz = ex - ox, ey - oy, ez - oz
    local c, s = cos(angle), sin(angle)
    local wx, wy, wz = ky * vz - kz * vy, kz * vx - kx * vz, kx * vy - ky * vx
    return ox + vx * c + wx * s, oy + vy * c + wy * s, oz + vz * c + wz * s
end

---Whether the arc, from its entry at time `tEntry`, keeps clear of every body as a chain of
---chords flown at the planning speed. The chords lie on the centre's side of the arc, which is
---the side of the body the corner goes around, so a body there is found a little early; a body
---beyond the arc can be missed by up to the chords' sagitta.
local function arcIsClear(ex, ey, ez, ox, oy, oz, kx, ky, kz, turn, radius, tEntry)
    local px, py, pz, t = ex, ey, ez, tEntry
    local step = turn / ARC_CHORDS
    local duration = radius * step / speed
    for j = 1, ARC_CHORDS do
        local qx, qy, qz = SystemPlanner.pointOnArc(ox, oy, oz, ex, ey, ez, kx, ky, kz, step * j)
        local dx, dy, dz = qx - px, qy - py, qz - pz
        local length = sqrt(dx * dx + dy * dy + dz * dz)
        if length > MIN_LENGTH then
            flatProfile(chordProfile, t, length)
            if firstHit(chordProfile, px, py, pz, dx / length, dy / length, dz / length, t, t + length / speed) >= 0 then
                return false
            end
        end
        px, py, pz, t = qx, qy, qz, t + duration
    end
    return true
end

---The arc radius and passing speed of a corner reached at time `tCorner`. The radius is the
---largest the legs and the turn-speed knee allow that the bodies leave clear, halving until one
---does. A corner that barely turns is passed at the planning speed with no arc; one with no clear
---arc is left sharp, to be passed at rest.
---@return number radius Zero for no arc
---@return number speed
local function roundCorner(px, py, pz, cx, cy, cz, nx, ny, nz, tCorner)
    local turn, la, lb = cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    if not turn or turn < MIN_ARC_ANGLE then return 0, speed end
    local tanHalf = tan(turn * HALF)
    local radius = min(kneeRadius, ARC_LEG_SHARE * min(la, lb) / tanHalf)
    for _ = 0, MAX_ARC_HALVINGS do
        if radius <= MIN_LENGTH then break end
        local ex, ey, ez, _, _, _, ox, oy, oz, kx, ky, kz = SystemPlanner.cornerArc(px, py, pz, cx, cy, cz, nx, ny, nz, radius)
        if not ex then break end
        if arcIsClear(ex, ey, ez, ox, oy, oz, kx, ky, kz, turn, radius, tCorner - radius * tanHalf / speed) then
            return radius, min(speed, SystemPlanner.turnSpeed(shipAccel, shipDrag, radius, shipTopSpeed))
        end
        radius = radius * HALF
    end
    return 0, 0
end

---The world point of waypoint `k` of the output (the start for -1): its offset from its frame
---body's position at the waypoint's time.
local function waypointPoint(k, sx, sy, sz)
    if k < 0 then return sx, sy, sz end
    local base = k * OUT_STRIDE
    local frame, x, y, z = out[base + OUT_FRAME], out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z]
    if frame == SYSTEM_FRAME then return x, y, z end
    local cx, cy, cz = bodyPosition(frame, out[base + OUT_T])
    return x + cx, y + cy, z + cz
end

---Gives every corner of the written plan its arc radius and passing speed. The goal keeps radius
---and speed zero, since the ship stops there. The plan's times stay as the legs gave them.
local function roundCorners(sx, sy, sz)
    for k = 0, written - 2 do
        local px, py, pz = waypointPoint(k - 1, sx, sy, sz)
        local cx, cy, cz = waypointPoint(k, sx, sy, sz)
        local nx, ny, nz = waypointPoint(k + 1, sx, sy, sz)
        local base = k * OUT_STRIDE
        out[base + OUT_RADIUS], out[base + OUT_SPEED] = roundCorner(px, py, pz, cx, cy, cz, nx, ny, nz, out[base + OUT_T])
    end
end

---Where the goal is at `time`: the point (gx, gy, gz) itself in the system frame, else that offset
---from the goal's frame body.
local function goalPoint(gx, gy, gz, time)
    if goalFrame == SYSTEM_FRAME then return gx, gy, gz end
    local cx, cy, cz = bodyPosition(goalFrame, time)
    return gx + cx, gy + cy, gz + cz
end

---One full plan toward the goal as it is at `goalTime`.
---@return boolean ok False when an endpoint is inside a body's hull
---@return number|nil tEnd The time the plan reaches its last point, nil when it stopped
local function planPass(sx, sy, sz, gx, gy, gz, goalTime)
    written, status, failures = 0, Status.Ok, 0
    resetHitRadii()
    local px, py, pz = goalPoint(gx, gy, gz, goalTime)
    if not (shrinkAround(sx, sy, sz, 0) and shrinkAround(px, py, pz, goalTime)) then return false end
    return true, planLeg(sx, sy, sz, 0, px, py, pz, goalFrame, 0)
end

---Whether the numbers and the frame index of a request are usable.
local function validRequest(sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit, outCapacity)
    local finite = isFinite(sx) and isFinite(sy) and isFinite(sz) and isFinite(gx) and isFinite(gy) and isFinite(gz)
    local frameValid = frame == math.floor(frame) and frame >= SYSTEM_FRAME and frame < bodyCount
    local shipValid = accel > 0 and isFinite(accel) and drag >= 0 and isFinite(drag)
    return finite and frameValid and shipValid and planSpeed > 0 and isFinite(planSpeed) and depthLimit >= 0
        and outCapacity >= 1 and bodyCount >= 0
end

---Sets the ship scratch from its forward acceleration and drag: the top speed, and the arc radius
---at which the turn speed reaches `TURN_SPEED_KNEE` of its limit `share · accel / drag`, which is
---`s² / (share · accel · √(1 − knee²))` for `s = knee · limit`. Both are infinite without drag.
local function setShip(accel, drag)
    shipAccel, shipDrag = accel, drag
    if drag > 0 then
        shipTopSpeed = accel / drag
        kneeRadius = TURN_SPEED_KNEE * TURN_SPEED_KNEE * TURN_THRUST_SHARE * accel
            / (drag * drag * sqrt(1 - TURN_SPEED_KNEE * TURN_SPEED_KNEE))
    else
        shipTopSpeed, kneeRadius = math.huge, math.huge
    end
end

---Sets the travel-drive scratch from the drive block, or turns the drive off for none. The drive
---needs a ship with drag (its braking law divides by it).
---@return boolean valid
local function setDrive(drive)
    driveOn = drive ~= nil
    if not driveOn then return true end
    local openCap, charge = drive[DRIVE_OPEN_CAP], drive[DRIVE_CHARGE]
    local rampUp, rampDown = drive[DRIVE_RAMP_UP], drive[DRIVE_RAMP_DOWN]
    local valid = isFinite(openCap) and openCap >= 1 and isFinite(charge) and charge >= 0
        and isFinite(rampUp) and rampUp > 0 and isFinite(rampDown) and rampDown > 0 and shipDrag > 0
    if not valid then
        driveOn = false
        return false
    end
    driveOpenCap, driveCharge, driveRampUp, driveRampDown = openCap, charge, rampUp, rampDown
    driveShortestLag = min(1 / rampUp, 1 / rampDown, 1 / shipDrag)
    driveBrakingTime = TravelDriveMath.brakingTime(rampDown, shipDrag)
    driveRelease = TravelDriveMath.HANDOVER_MULT * driveBrakingTime * speed
    return true
end

---Plans a path from the start to the goal through the bodies. The goal is the point (gx, gy, gz)
---in the system frame, or that offset (world axes) from the centre of body `frame`; a goal on a
---moving body is planned for the time the ship gets there. With a drive block, a leg is flown
---under the travel drive where that is sooner, and `Out.Drive` marks it; `planSpeed` is then the
---ship's speed without the drive.
---@param bodiesBuffer userdata `double[?]`, `bodyCount * BODY_STRIDE` values
---@param count integer Number of bodies
---@param sx number Start x
---@param sy number Start y
---@param sz number Start z
---@param frame integer Index of the goal's frame body, or `SYSTEM_FRAME`
---@param gx number Goal x
---@param gy number Goal y
---@param gz number Goal z
---@param planSpeed number Speed the path is flown at, above zero; sets the times in the output
---@param accel number The ship's forward acceleration, above zero; sets the corner speeds
---@param drag number The ship's linear drag rate, zero or more; sets the corner speeds
---@param depthLimit integer Detours nested at most this deep, zero or more
---@param outBuffer userdata `double[?]`, `outCapacity * OUT_STRIDE` values
---@param outCapacity integer Waypoints the output buffer holds
---@param driveBuffer userdata|nil `double[?]`, `DRIVE_STRIDE` values, nil for no drive
---@return integer status A `Status` value
---@return integer waypointCount Waypoints written, the goal last when the status allows a plan
function SystemPlanner.plan(bodiesBuffer, count, sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit,
                            outBuffer, outCapacity, driveBuffer)
    bodies, bodyCount, speed, maxDepth, out, capacity = bodiesBuffer, count, planSpeed, depthLimit, outBuffer, outCapacity
    goalFrame, written, status, failures = frame, 0, Status.Ok, 0
    if not validRequest(sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit, outCapacity) then
        return Status.InvalidInput, 0
    end
    setShip(accel, drag)
    if not setDrive(driveBuffer) or not prepareBodies() then return Status.InvalidInput, 0 end
    if frame ~= SYSTEM_FRAME and speedBound[frame + 1] >= planSpeed then return Status.GoalOutrun, 0 end

    local fx, fy, fz = goalPoint(gx, gy, gz, 0)
    local dx, dy, dz = fx - sx, fy - sy, fz - sz
    local goalTime = sqrt(dx * dx + dy * dy + dz * dz) / planSpeed
    local settled = false
    for _ = 1, frame == SYSTEM_FRAME and 1 or MAX_GOAL_ITERATIONS do
        local ok, tEnd = planPass(sx, sy, sz, gx, gy, gz, goalTime)
        if not ok then return Status.EndpointInsideBody, 0 end
        settled = frame == SYSTEM_FRAME or tEnd == nil or abs(tEnd - goalTime) <= GOAL_TIME_TOLERANCE
        if settled then break end
        goalTime = tEnd
    end
    if not settled and status == Status.Ok then status = Status.GoalNotSettled end

    -- The goal keeps the caller's offset exactly, not the one rounded from the last pass's point.
    if frame ~= SYSTEM_FRAME and written > 0 and status ~= Status.CapacityExceeded then
        local base = (written - 1) * OUT_STRIDE
        out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z] = gx, gy, gz
    end
    if status ~= Status.CapacityExceeded then roundCorners(sx, sy, sz) end
    return status, written
end

return SystemPlanner
