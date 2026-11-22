--- System planner kernel: a path through a system of orbiting bodies as waypoints (a body index and
--- an offset from it). Straight legs get detours around the bodies' safe spheres at their predicted
--- positions, corners are rounded into arcs, legs are timed under the travel drive where that is
--- sooner (a gentle detour corner keeps the drive run), a goal inside its body's well is flown in the body's frame from the well's edge, and a
--- cost zone is gone around where that is cheaper than crossing it.
--- Pure numbers in and out (no entities, no globals), so it runs on a worker or in a test state.
--- Not reentrant: it keeps module scratch state.
--- Design: doc/script/nav/system-planner.md.
local OrbitMath       = require("Shared.Helpers.OrbitMath")
local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")

local sqrt, atan2, acos, sin, cos, tan, exp, log = math.sqrt, math.atan2, math.acos, math.sin, math.cos, math.tan, math.exp, math.log
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
    ---The goal's frame body can move as fast as the ship can fly to the body's well (or, for a
    ---goal outside a well, to the goal), so the ship may never catch it.
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
-- waypoint is flown under the travel drive, which is released before the waypoint. `through` is 1
-- when the drive is not released there: the leg that ends at the waypoint and the leg after it are
-- one drive run, and the waypoint's speed is the run's.
local OUT_FRAME        = 0
local OUT_X            = 1
local OUT_Y            = 2
local OUT_Z            = 3
local OUT_T            = 4
local OUT_RADIUS       = 5
local OUT_SPEED        = 6
local OUT_DRIVE        = 7
local OUT_THROUGH      = 8
local OUT_STRIDE       = 9
local SYSTEM_FRAME     = -1
-- Drive buffer: DRIVE_STRIDE doubles, in the order of these offsets. The multiplier in open
-- space, the seconds the drive charges at normal speed, and the rates at which its multiplier
-- rises (per second, times the multiplier above 1) and falls (the rate of an exponential), and
-- the length below which a leg is never flown under the drive.
local DRIVE_OPEN_CAP   = 0
local DRIVE_CHARGE     = 1
local DRIVE_RAMP_UP    = 2
local DRIVE_RAMP_DOWN  = 3
local DRIVE_MIN_LENGTH = 4
local DRIVE_STRIDE     = 5
-- Zone buffer: ZONE_STRIDE doubles per cost zone, in the order of these offsets. A zone is a sphere
-- of `radius` at the offset (x, y, z) from the centre of body `frame` (the point itself for
-- SYSTEM_FRAME), and each metre travelled inside costs `cost` extra. A zone has no hull: a leg is
-- bent around it only where that is cheaper than crossing it, and it never fails a plan.
local ZONE_FRAME       = 0
local ZONE_X           = 1
local ZONE_Y           = 2
local ZONE_Z           = 3
local ZONE_RADIUS_AT   = 4
local ZONE_COST        = 5
local ZONE_STRIDE      = 6

SystemPlanner.BODY_STRIDE = BODY_STRIDE
SystemPlanner.Body = {
    Parent = PARENT, X = POS_X, Y = POS_Y, Z = POS_Z, OrbitRadius = ORBIT_RADIUS, Phase = PHASE,
    AngularSpeed = ANGULAR_SPEED, Inclination = INCLINATION, Radius = BODY_RADIUS, SafeRadius = SAFE_RADIUS,
    ZoneRadius = ZONE_RADIUS, ZoneCap = ZONE_CAP,
}
SystemPlanner.OUT_STRIDE = OUT_STRIDE
SystemPlanner.Out = {
    Frame = OUT_FRAME, X = OUT_X, Y = OUT_Y, Z = OUT_Z, T = OUT_T, Radius = OUT_RADIUS, Speed = OUT_SPEED,
    Drive = OUT_DRIVE, Through = OUT_THROUGH,
}
SystemPlanner.DRIVE_STRIDE = DRIVE_STRIDE
SystemPlanner.Drive = {
    OpenCap = DRIVE_OPEN_CAP, Charge = DRIVE_CHARGE, RampUp = DRIVE_RAMP_UP, RampDown = DRIVE_RAMP_DOWN,
    MinLength = DRIVE_MIN_LENGTH,
}
SystemPlanner.ZONE_STRIDE = ZONE_STRIDE
SystemPlanner.Zone = {
    Frame = ZONE_FRAME, X = ZONE_X, Y = ZONE_Y, Z = ZONE_Z, Radius = ZONE_RADIUS_AT, Cost = ZONE_COST,
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
-- A point this close (a fraction of the radius) outside a safe sphere counts as on it, so that an
-- endpoint on the surface is inside whatever the rounding of the positions it is measured from.
local ENDPOINT_TOUCH   = 1e-6
-- Distances below this make a direction undefined.
local MIN_LENGTH       = 1e-9
local HALF             = 0.5
-- A goal on a moving body is re-planned for the time the last plan arrives at it, until that time
-- changes by less than the tolerance (seconds) or the iteration limit is reached. Each pass
-- shrinks the error by the goal's speed over the planning speed. The tolerance is above the
-- jitter that the stepped overlap search puts on a detour point (about 1e-5 s).
local GOAL_TIME_TOLERANCE = 1e-4
local MAX_GOAL_ITERATIONS = 50
-- A goal inside its body's well is planned from an entry point this share inside the well's radius,
-- so a ship that arrives a little off the entry is still inside the well that carries it.
local WELL_ENTRY_MARGIN = 0.05
-- How a goal on a body relates to the body's well: not in it (or it has none), a ship that starts
-- inside it, or one that starts outside and has to enter it.
local WELL_NONE, WELL_INSIDE, WELL_ENTRY = 0, 1, 2

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
SystemPlanner.WELL_ENTRY_MARGIN = WELL_ENTRY_MARGIN

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

-- A corner flown under the drive is a turn of the velocity toward the new leg with the drag's time
-- constant. The nose turns one time constant before the corner (the lead). The turn counts as done
-- when the velocity is off the new heading by at most this share of the run's speed (the error
-- starts at `2 · sin(turn / 2)` of the speed and falls by `e^-τ`), and it is checked against the
-- bodies as this many chords.
local DRIVE_TURN_LEAD_TAUS  = 1
local DRIVE_TURN_TOLERANCE  = 0.01
local DRIVE_TURN_CHORDS     = 12

-- The exit of a cost zone is found within a quarter radius by the overlap walk, then bisected this
-- many times: the length inside the zone is what the cross-or-go-around rule weighs, so it needs
-- more than the walk's step.
local ZONE_EXIT_BISECTIONS = 20

-- Scratch state of the running plan.
local bodies, bodyCount, speed, maxDepth, out, capacity, written, status, failures, goalFrame
-- The cost zones: their buffer and count. Obstacles are numbered bodies first (0 .. bodyCount - 1),
-- then zones (bodyCount .. obstacleCount - 1), so the hit test and the detour work on both.
local zones, zoneCount, obstacleCount = nil, 0, 0
-- Per zone: whether this plan ignores it (it costs nothing, or it holds an endpoint). Per recursion
-- depth: the zones left out of the hit test of the leg being planned (ignored, or waived because
-- crossing is cheaper than going around).
local zoneIgnored, waivedByDepth = {}, {}
-- The ship's forward acceleration and drag, its top speed (infinite without drag) and the arc
-- radius at the turn-speed knee (infinite without drag).
local shipAccel, shipDrag, shipTopSpeed, kneeRadius
-- The travel drive: whether it is planned, the multiplier in open space, the charge time, the
-- rise and fall rates, the shortest time constant of the drive and the ship's speed, the braking
-- law's time constant and the distance from a leg's end at which the drive is released, and the
-- length below which a leg is not driven.
local driveOn, driveOpenCap, driveCharge, driveRampUp, driveRampDown = false, 1, 0, 1, 1
local driveMinLength = 0
local driveShortestLag, driveBrakingTime, driveRelease = 1, 1, 0
-- Per body: the speed bound of its orbit chain, and the radius of the sphere the hit test uses.
local speedBound, hitRadius = {}, {}
-- The frame the legs being planned are flown in: a body index (the legs are straight lines of
-- offsets from that body, which carries the ship), or SYSTEM_FRAME. Per body: the speed bound of
-- its motion relative to that frame. Per waypoint written: the frame of the leg that ends there.
local legFrame, legBound, legFrameOf = SYSTEM_FRAME, {}, {}
-- A goal on a body: how it relates to the body's well, the entry sphere's radius, the waypoint
-- written for the entry, and the entry's offset from the body and its distance from the start in
-- the last pass.
local wellMode, entryRadius, entryIndex, entryChord = WELL_NONE, 0, -1, 0
-- The drive multiplier and speed at the end of the last leg written with `Through`: the state a
-- run carries through the entry. Read only when the entry has `Through`.
local runEndMultiplier, runEndSpeed = 1, 0
-- How close to the goal the ship counts as arrived, and the timeline of the leg written last (nil
-- for a leg too short to time): the goal's time is read from that timeline.
local arrivalRadius, lastProfile = 0, nil
-- The distance a leg that ends at the goal stops short of it, and the goal point in the coordinates
-- of the legs being planned (`goalStop`). A table, for the limit on a chunk's locals.
local goalCut = { standOff = 0, x = 0, y = 0, z = 0 }
-- For a leg that ends at the well's entry (`legEnd`): the goal's offset from its body (x, y, z), the
-- entry's time, and the offset the plan's entry has from the body (offsetX, offsetY, offsetZ). A
-- table, for the limit on a chunk's locals.
local entryCut = { x = 0, y = 0, z = 0, time = 0, offsetX = 0, offsetY = 0, offsetZ = 0 }
-- How a leg ends, when it does not end at the point it is given: at the goal's stop point, or at the
-- well's entry (`legEnd`). The tail of a detour on the leg ends as the leg does.
local END = { goal = 1, entry = 2 }
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

---Checks the zone buffer: every zone is on a body or in the system, with finite numbers, a radius
---above zero and a cost of zero or more. Needs `prepareBodies` first.
---@return boolean valid
local function prepareZones()
    for z = 0, zoneCount - 1 do
        local base = z * ZONE_STRIDE
        local frame = zones[base + ZONE_FRAME]
        local frameValid = frame == math.floor(frame) and frame >= SYSTEM_FRAME and frame < bodyCount
        local centreValid = isFinite(zones[base + ZONE_X]) and isFinite(zones[base + ZONE_Y])
            and isFinite(zones[base + ZONE_Z])
        local radius, cost = zones[base + ZONE_RADIUS_AT], zones[base + ZONE_COST]
        if not (frameValid and centreValid and isFinite(radius) and radius > 0 and isFinite(cost) and cost >= 0) then
            return false
        end
    end
    return true
end

---Restores every body's hit radius to its safe radius, undoing the endpoint shrinks of a pass, and
---every zone's to its radius.
local function resetHitRadii()
    for i = 0, bodyCount - 1 do
        hitRadius[i + 1] = bodies[i * BODY_STRIDE + SAFE_RADIUS]
    end
    for z = 0, zoneCount - 1 do
        hitRadius[bodyCount + z + 1] = zones[z * ZONE_STRIDE + ZONE_RADIUS_AT]
    end
end

---Shrinks the hit radius of every body whose safe sphere holds the point at time `t`, a point on
---the sphere's surface included (a leg that ends there counts as inside it).
---@return boolean ok False when the point is inside a body's hull
local function shrinkAround(x, y, z, t)
    for i = 0, bodyCount - 1 do
        local cx, cy, cz = bodyPosition(i, t)
        local dx, dy, dz = x - cx, y - cy, z - cz
        local d = sqrt(dx * dx + dy * dy + dz * dz)
        local radius = bodies[i * BODY_STRIDE + BODY_RADIUS]
        if d <= radius then return false end
        if d <= hitRadius[i + 1] * (1 + ENDPOINT_TOUCH) then
            hitRadius[i + 1] = max(radius, d * (1 - ENDPOINT_SHRINK))
        end
    end
    return true
end

---The bound of the motion of something that rides on body `owner` (or on the system for
---SYSTEM_FRAME) relative to the legs' frame: at most the sum of the two bounds, and zero for
---something that rides on the legs' frame body itself.
local function boundRelativeToLegs(owner)
    local own = owner == SYSTEM_FRAME and 0 or speedBound[owner + 1]
    if legFrame == SYSTEM_FRAME then return own end
    if owner == legFrame then return 0 end
    return own + speedBound[legFrame + 1]
end

---Flies the legs that follow in the frame of body `frame`, or in the system for SYSTEM_FRAME.
---Sets the bound of each obstacle's motion relative to that frame: a zone moves with the body it
---rides on.
local function setLegFrame(frame)
    legFrame = frame
    for i = 0, bodyCount - 1 do
        legBound[i + 1] = boundRelativeToLegs(i)
    end
    for z = 0, zoneCount - 1 do
        legBound[bodyCount + z + 1] = boundRelativeToLegs(zones[z * ZONE_STRIDE + ZONE_FRAME])
    end
end

---Where zone `z` is `t` seconds from the snapshot, in the system: its offset from the body it rides
---on, at that body's position at `t`.
local function zonePosition(z, t)
    local base = z * ZONE_STRIDE
    local x, y, zz = zones[base + ZONE_X], zones[base + ZONE_Y], zones[base + ZONE_Z]
    local frame = zones[base + ZONE_FRAME]
    if frame == SYSTEM_FRAME then return x, y, zz end
    local cx, cy, cz = bodyPosition(frame, t)
    return x + cx, y + cy, zz + cz
end

---Where obstacle `i` (a body, or a zone from `bodyCount` on) is `t` seconds from the snapshot, in
---the legs' frame: the offset from the frame body's position at `t`, or the system position for
---the system frame.
local function legBodyPosition(i, t)
    local x, y, z
    if i >= bodyCount then x, y, z = zonePosition(i - bodyCount, t) else x, y, z = bodyPosition(i, t) end
    if legFrame == SYSTEM_FRAME then return x, y, z end
    local fx, fy, fz = bodyPosition(legFrame, t)
    return x - fx, y - fy, z - fz
end

---The frame a waypoint of a detour around obstacle `i` is stored in: the body itself, or a zone's
---frame. A zone in the system, passed inside a well's frame, is stored in that well's frame, where
---the leg's coordinates are offsets.
local function detourFrame(i)
    if i < bodyCount then return i end
    local frame = zones[(i - bodyCount) * ZONE_STRIDE + ZONE_FRAME]
    if frame == SYSTEM_FRAME then return legFrame end
    return frame
end

---Sets which zones this plan ignores: one that costs nothing, and one that holds the start (at
---time zero) or the goal (at `goalTime`), since entering that one cannot be avoided.
local function markIgnoredZones(sx, sy, sz, gx, gy, gz, goalTime)
    for z = 0, zoneCount - 1 do
        local radius = zones[z * ZONE_STRIDE + ZONE_RADIUS_AT] * (1 + ENDPOINT_TOUCH)
        local zx, zy, zz = zonePosition(z, 0)
        local startHeld = (sx - zx) ^ 2 + (sy - zy) ^ 2 + (sz - zz) ^ 2 <= radius * radius
        zx, zy, zz = zonePosition(z, goalTime)
        local goalHeld = (gx - zx) ^ 2 + (gy - zy) ^ 2 + (gz - zz) ^ 2 <= radius * radius
        zoneIgnored[z + 1] = zones[z * ZONE_STRIDE + ZONE_COST] <= 0 or startHeld or goalHeld
    end
end

---The zones left out of the hit test of a leg at recursion depth `depth`: the ignored ones, to
---which the leg adds those it waives. Nil without zones, so a plan without zones tests nothing.
---@return table<integer, boolean>|nil
local function legWaived(depth)
    if zoneCount == 0 then return nil end
    local waived = waivedByDepth[depth + 1]
    if not waived then
        waived = {}
        waivedByDepth[depth + 1] = waived
    end
    for i = 0, bodyCount - 1 do waived[i] = false end
    for z = 0, zoneCount - 1 do waived[bodyCount + z] = zoneIgnored[z + 1] end
    return waived
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
    profile.count, profile.drive, profile.through = 2, false, false
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
            local cx, cy, cz = legBodyPosition(i, t)
            local dx, dy, dz = x - cx, y - cy, z - cz
            local d = sqrt(dx * dx + dy * dy + dz * dz)
            limit = min(limit, TravelDriveMath.zoneCap(driveOpenCap, bodies[base + ZONE_CAP], zoneRadius, d))
            local influence = zoneRadius * TravelDriveMath.INFLUENCE_RADIUS_FACTOR
            local gap = max(abs(d - influence), DRIVE_FEATURE_FLOOR * influence)
            step = min(step, DRIVE_FEATURE_STEP * gap / (v + legBound[i + 1]))
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
---A leg that continues a run starts from the multiplier `carryM` and speed `carryV` with no
---charge. A leg after which the run goes on for `beyond` more length (above zero) is not
---released: it ends at the end of the leg with the multiplier and speed it has there, and the
---braking law and the step bound see the distance left to the end of the run.
---False for a leg too short to charge and release, or one that needs more than `MAX_DRIVE_STEPS`.
local function driveProfile(profile, t0, length, ax, ay, az, ux, uy, uz, beyond, carryM, carryV)
    local continues = beyond > 0
    local sEnd = continues and length or length - driveRelease
    local t, s, m, v = t0 + driveCharge, speed * driveCharge, 1, speed
    if carryM then t, s, m, v = t0, 0, carryM, carryV end
    if s >= sEnd then return false end
    local times, along, vmax = profile.t, profile.s, profile.vmax
    times[1], along[1] = t0, 0
    local count = 1
    if not carryM and driveCharge > 0 then
        times[2], along[2], vmax[1], count = t, s, speed, 2
    end
    for _ = 1, MAX_DRIVE_STEPS do
        local limit, dt = zoneState(ax + ux * s, ay + uy * s, az + uz * s, t, v)
        local runLeft = length - s + beyond
        local wanted = TravelDriveMath.brakingMultiplier(runLeft, speed, driveBrakingTime)
        local target = max(1, min(limit, wanted))
        dt = min(dt, DRIVE_FEATURE_STEP * runLeft / v)
        local offTarget = abs(m - target) > DRIVE_TRANSIENT_TOLERANCE * target
        local lagging = abs(v - speed * m) > DRIVE_TRANSIENT_TOLERANCE * v
        if offTarget or lagging then
            dt = min(dt, DRIVE_TRANSIENT_STEP * driveShortestLag)
        end
        dt = max(dt, DRIVE_MIN_STEP_S)
        local mNew, vNew = driveStep(m, v, target, dt)
        local sNew = s + HALF * (v + vNew) * dt
        local done = sNew >= sEnd
        if done then
            dt = dt * (sEnd - s) / (sNew - s)
            sNew = sEnd
            -- The state the run carries on from is the one at the clipped time.
            if continues then mNew, vNew = driveStep(m, v, target, dt) end
        end
        count = count + 1
        times[count], along[count], vmax[count - 1] = t + dt, sNew, max(v, vNew)
        t, s, m, v = t + dt, sNew, mNew, vNew
        if done then
            if continues then
                profile.count, profile.drive, profile.through = count, true, true
                profile.endM, profile.endV = m, v
            else
                times[count + 1], along[count + 1], vmax[count] = t + (length - s) / speed, length, speed
                profile.count, profile.drive, profile.through = count + 1, true, false
            end
            return true
        end
    end
    return false
end

---Fills `profile` with the leg of `length` from `a` along `u` from time `t0`: under the drive when
---that gets to the end sooner, else at the planning speed. A leg shorter than the drive's minimum
---length is not timed under the drive, unless it continues a run. `run` (or nil) describes the
---run the leg belongs to: `beyond`, the length the run goes on past the leg, and `m` and `v`, the
---drive multiplier and speed it starts from when it continues a run.
local function buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run)
    local carried = run and run.m
    if driveOn and (carried or length >= driveMinLength) then
        local timed = driveProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run and run.beyond or 0, carried,
            run and run.v)
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

---The time the ship is `along` from the start of the leg, linear between the samples of the
---profile; the leg's start for a distance of zero or less, its end for one past the leg.
local function timeAtDistance(profile, along)
    local times, dists = profile.t, profile.s
    local k, last = 1, profile.count - 1
    while k < last and dists[k + 1] < along do k = k + 1 end
    local s1, s2 = dists[k], dists[k + 1]
    if s2 <= s1 then return times[k] end
    return times[k] + (times[k + 1] - times[k]) * min(1, max(0, (along - s1) / (s2 - s1)))
end

---The ship's mean speed over the interval of the profile that holds the distance `along` from the
---start of the leg (the first interval for a distance of zero or less, the last one past the leg).
local function speedAtDistance(profile, along)
    local times, dists = profile.t, profile.s
    local k, last = 1, profile.count - 1
    while k < last and dists[k + 1] < along do k = k + 1 end
    local dt = times[k + 1] - times[k]
    if dt <= 0 then return 0 end
    return (dists[k + 1] - dists[k]) / dt
end

---The distance from the ship, `t` seconds in (interval `k` of the profile), to the centre of body `i`.
local function gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t)
    local along = distanceAt(profile, t, k)
    local cx, cy, cz = legBodyPosition(i, t)
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
    local radius, bound = hitRadius[i + 1], legBound[i + 1]
    local t, k = t0, 1
    while t <= tLimit do
        k = locate(profile, t, k)
        local d = gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t)
        if d <= radius then return t end
        t = stepTo(profile, k, t, max(d - radius, GRAZE_STEP * radius) / (profile.vmax[k] + bound))
    end
    return nil
end

---The first obstacle (a body, or a zone from `bodyCount` on) whose sphere the leg enters, and
---when, among the obstacles below `limit` that are not in `waived` (nil for none). The leg runs
---from `a` along `u` through the profile's times, from `t0` to `tEnd`.
---@return integer obstacle Index, -1 for none
---@return number time
local function firstHit(profile, ax, ay, az, ux, uy, uz, t0, tEnd, limit, waived)
    local found, foundTime = -1, tEnd
    for i = 0, limit - 1 do
        if not (waived and waived[i]) then
            local hit = bodyHit(profile, i, ax, ay, az, ux, uy, uz, t0, foundTime)
            if hit then found, foundTime = i, hit end
        end
    end
    return found, foundTime
end

---Walks through the overlap of the leg with obstacle `i`'s sphere, which starts at `tIn`: it steps
---by a share of the radius, and after each exit looks ahead for a re-entry.
---@return number last The last time sampled inside the sphere
---@return number|nil outside The time sampled outside after it, nil when the leg ends inside
local function overlapWalk(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    local radius, bound = hitRadius[i + 1], legBound[i + 1]
    local last, t, k = tIn, tIn, 1
    while t <= tEnd do
        k = locate(profile, t, k)
        if gapAt(profile, k, i, ax, ay, az, ux, uy, uz, t) <= radius then
            last = t
            t = stepTo(profile, k, t, OVERLAP_STEP * radius / (profile.vmax[k] + bound))
        else
            local reentry = bodyHit(profile, i, ax, ay, az, ux, uy, uz, t, tEnd)
            if not reentry then return last, t end
            t = reentry
        end
    end
    return last, nil
end

---The time in the overlap that is half way between its start and its end. The overlap ends at
---the last time the walk finds the leg inside the sphere.
local function overlapMidTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    local last = overlapWalk(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    return (tIn + last) * HALF
end

---The time the leg leaves obstacle `i`'s sphere for good, which it entered at `tIn`: the walk's
---last inside sample, bisected toward the first outside sample after it. A leg that ends inside
---leaves at its end.
local function overlapExitTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    local last, outside = overlapWalk(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    if not outside then return tEnd end
    local radius = hitRadius[i + 1]
    local lo, hi, k0 = last, outside, locate(profile, last, 1)
    for _ = 1, ZONE_EXIT_BISECTIONS do
        local mid = (lo + hi) * HALF
        if gapAt(profile, locate(profile, mid, k0), i, ax, ay, az, ux, uy, uz, mid) <= radius then
            lo = mid
        else
            hi = mid
        end
    end
    return (lo + hi) * HALF
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
    local delta = OVERLAP_STEP * hitRadius[i + 1] / (speed + legBound[i + 1])
    local x1, y1, z1 = legBodyPosition(i, tm + delta)
    local x0, y0, z0 = legBodyPosition(i, tm - delta)
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
---A positive `push` widens the detour sphere by that much, within the cap that keeps it clear of
---the endpoints; when the cap leaves the sphere no wider, there is no point (nil).
---@return number|nil x
---@return number y
---@return number z
local function detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tEnd, flip, push)
    local tm = overlapMidTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
    local cx, cy, cz = legBodyPosition(i, tm)
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
    local cap = hit + (nearest - hit) * HALF
    local margin = hit * (1 + DETOUR_MARGIN)
    local r = min(margin, cap)
    if push then
        local pushed = min(margin + push, cap)
        if pushed <= r then return nil, 0, 0 end
        r = pushed
    end

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

---Writes the point (x, y, z) of the legs' frame reached at time `t` as an offset from the output
---frame body's position at `t` (the point itself for the system frame), reached by a leg with the
---timeline `profile` (nil for a leg that is not timed), and notes the legs' frame. Returns false
---(and sets the status) when the buffer is full.
local function append(x, y, z, t, frame, profile)
    if written >= capacity then
        status = Status.CapacityExceeded
        return false
    end
    if frame ~= SYSTEM_FRAME then
        local cx, cy, cz = legBodyPosition(frame, t)
        x, y, z = x - cx, y - cy, z - cz
    end
    legFrameOf[written + 1] = legFrame
    local base = written * OUT_STRIDE
    out[base + OUT_FRAME], out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z], out[base + OUT_T] =
        frame, x, y, z, t
    out[base + OUT_RADIUS], out[base + OUT_SPEED] = 0, 0
    out[base + OUT_DRIVE] = profile and profile.drive and 1 or 0
    out[base + OUT_THROUGH] = profile and profile.through and 1 or 0
    if profile and profile.through then
        runEndMultiplier, runEndSpeed = profile.endM, profile.endV
        -- A waypoint the run goes through is passed at the run's speed there.
        out[base + OUT_SPEED] = profile.endV
    end
    lastProfile = profile
    written = written + 1
    return true
end

---Records that a leg could not be cleared, keeping the first reason as the status.
local function fail(reason)
    failures = failures + 1
    if status == Status.Ok then status = reason end
end

local planLeg

---Where a leg that ends at the goal and starts at (ax, ay, az) ends: `goalCut.standOff` short of the
---goal point, on the line from the goal to the start. A start already that close ends where it is.
local function goalStop(ax, ay, az)
    local dx, dy, dz = ax - goalCut.x, ay - goalCut.y, az - goalCut.z
    local d = sqrt(dx * dx + dy * dy + dz * dz)
    if d <= goalCut.standOff then return ax, ay, az end
    local share = goalCut.standOff / d
    return goalCut.x + dx * share, goalCut.y + dy * share, goalCut.z + dz * share
end

---The offset from the goal's body, at `entryTime`, of the point where the straight line from the
---start to the goal first crosses the entry sphere: the start is `a` from the body and the goal is
---`g`, so the line is `a + λ(g − a)` and `λ` is the smaller root of `|a + λ(g − a)|² = r²`. A start
---already inside the sphere at that time enters at once (`λ = 0`).
local function entryOffset(sx, sy, sz, gx, gy, gz, entryTime)
    local cx, cy, cz = bodyPosition(goalFrame, entryTime)
    local ax, ay, az = sx - cx, sy - cy, sz - cz
    local dx, dy, dz = gx - ax, gy - ay, gz - az
    local qa = dx * dx + dy * dy + dz * dz
    local qb = ax * dx + ay * dy + az * dz
    local qc = ax * ax + ay * ay + az * az - entryRadius * entryRadius
    local lambda = 0
    if qc > 0 and qa > MIN_LENGTH then
        lambda = max(0, (-qb - sqrt(max(0, qb * qb - qa * qc))) / qa)
    end
    return ax + dx * lambda, ay + dy * lambda, az + dz * lambda
end

---Where a leg of end rule `rule` (`END.goal` or `END.entry`) that starts at (ax, ay, az) ends: the
---goal's stop point, or the well's entry, each on the line from the leg's own start to the goal.
local function legEnd(rule, ax, ay, az)
    if rule == END.goal then return goalStop(ax, ay, az) end
    local ox, oy, oz = entryOffset(ax, ay, az, entryCut.x, entryCut.y, entryCut.z, entryCut.time)
    local cx, cy, cz = bodyPosition(goalFrame, entryCut.time)
    return ox + cx, oy + cy, oz + cz
end

---The run of the first part of a leg that a detour splits: it starts as the leg does, and it ends
---at the detour's corner, where a run ends. Nil for a leg without a run.
local function headRun(run)
    if not (run and run.m) then return nil end
    return { beyond = 0, m = run.m, v = run.v }
end

---The run of the last part of a leg that a detour splits: it ends as the leg does, and starts at
---the detour's corner, so it starts a run and doesn't continue one.
local function tailRun(run)
    if not (run and run.beyond > 0) then return nil end
    return { beyond = run.beyond }
end

---Whether a leg a to b that enters cost zone `i` at `tIn` is cheaper around it than through it:
---the detour's extra length is less than the zone's cost times the length of the leg inside it,
---from the entry to the last exit. A zone with no detour point is crossed.
local function zoneWorthDetour(profile, i, ax, ay, az, bx, by, bz, length, ux, uy, uz, tIn, tB)
    local wx, wy, wz = detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tB, false)
    if not wx then return false end
    local viaLength = sqrt((wx - ax) ^ 2 + (wy - ay) ^ 2 + (wz - az) ^ 2)
        + sqrt((bx - wx) ^ 2 + (by - wy) ^ 2 + (bz - wz) ^ 2)
    local tOut = overlapExitTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tB)
    local inside = distanceAt(profile, tOut, locate(profile, tOut, 1))
        - distanceAt(profile, tIn, locate(profile, tIn, 1))
    return viaLength - length < zones[(i - bodyCount) * ZONE_STRIDE + ZONE_COST] * inside
end

local driveCornerHolds, legWaypointPoint

---Whether a detour a to w to b may keep a drive run through the corner w: the drive is planned, a
---run starts at or goes on through the detour's head, and a run is left after the corner (`rest`
---long, or the run goes on past b by `before`).
---@return boolean
---@return number rest The distance from w to b
---@return number before The length the run goes on past b
local function detourMayKeepRun(ax, ay, az, wx, wy, wz, bx, by, bz, run)
    local rest = sqrt((bx - wx) ^ 2 + (by - wy) ^ 2 + (bz - wz) ^ 2)
    local before = run and run.beyond or 0
    local startsRun = run and run.m or sqrt((wx - ax) ^ 2 + (wy - ay) ^ 2 + (wz - az) ^ 2) >= driveMinLength
    return driveOn and startsRun and (before > 0 or rest > driveRelease), rest, before
end

---The push of a detour's waypoint `w` around body `i`, or nil for none: the turn's cut, when the
---corner at `w` is expected to fail only because its turn cuts into body `i`. The corner is
---estimated from the leg a to b's own timeline (`profile`): the run's speed and time at the
---distance of `w` along it, with the leg's start and end as the points before and after the corner.
---@return number|nil push Metres
local function predictedPush(profile, i, ax, ay, az, wx, wy, wz, bx, by, bz, run)
    if not (profile.drive and detourMayKeepRun(ax, ay, az, wx, wy, wz, bx, by, bz, run)) then return nil end
    local along = sqrt((wx - ax) ^ 2 + (wy - ay) ^ 2 + (wz - az) ^ 2)
    local v = speedAtDistance(profile, along)
    if v <= 0 then return nil end
    local holds, blocker, cut = driveCornerHolds(ax, ay, az, wx, wy, wz, bx, by, bz, v, timeAtDistance(profile, along))
    if holds or blocker ~= i then return nil end
    return cut
end

---Plans the head a to w of a detour as a drive run that goes on through the detour corner w, and
---the tail w to b as its continuation, and keeps both when the corner holds the drive
---(`driveCornerHolds`), judged on the first waypoint the tail wrote. The head is planned first,
---since the corner's speed is the run's speed there; a corner that does not hold leaves everything
---as it was.
---@return boolean kept Whether the plan of the detour was written
---@return number|nil tEnd The time at b, nil when the plan stopped
local function planDetourThrough(ax, ay, az, t0, wx, wy, wz, frame, bx, by, bz, bFrame, depth, run, endRule)
    local tx, ty, tz = bx, by, bz
    if endRule then tx, ty, tz = legEnd(endRule, wx, wy, wz) end
    local mayKeep, rest, before = detourMayKeepRun(ax, ay, az, wx, wy, wz, tx, ty, tz, run)
    if not mayKeep then return false end
    local savedWritten, savedFailures, savedStatus = written, failures, status
    local tW = planLeg(ax, ay, az, t0, wx, wy, wz, frame, depth + 1,
        { beyond = rest + before, m = run and run.m, v = run and run.v })
    if not tW then return true, nil end
    local corner = written - 1
    if out[corner * OUT_STRIDE + OUT_THROUGH] ~= 0 then
        local cornerSpeed = out[corner * OUT_STRIDE + OUT_SPEED]
        local tEnd = planLeg(wx, wy, wz, tW, bx, by, bz, bFrame, depth + 1,
            { beyond = before, m = runEndMultiplier, v = runEndSpeed }, endRule)
        if not tEnd then return true, nil end
        local px, py, pz = ax, ay, az
        if corner - 1 >= savedWritten then px, py, pz = legWaypointPoint(corner - 1, 0, 0, 0) end
        local nx, ny, nz = legWaypointPoint(corner + 1, 0, 0, 0)
        if out[(corner + 1) * OUT_STRIDE + OUT_DRIVE] ~= 0
            and driveCornerHolds(px, py, pz, wx, wy, wz, nx, ny, nz, cornerSpeed, tW) then
            return true, tEnd
        end
    end
    written, failures, status = savedWritten, savedFailures, savedStatus
    return false
end

---Plans the leg a to b, whose timeline is `profile`, through one detour around obstacle `i` and
---writes its waypoints; the detour waypoint is in the obstacle's frame (`detourFrame`), and b in
---`bFrame`. `flip` passes the obstacle on the other side. `run` is the leg's run (see
---`buildProfile`). The drive run goes through the detour's corner when that holds on the plan of
---its tail (`planDetourThrough`), else the corner ends it at the detour's own waypoint. A corner
---expected to fail only because its turn cuts into the obstacle (`predictedPush`) is tried with the
---waypoint pushed out by the turn's cut.
---@return number|nil tEnd The time at b, nil when the plan stopped
local function planDetour(profile, i, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0, tIn, tB, depth, flip, run, endRule)
    local wx, wy, wz = detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tB, flip)
    if not wx then
        fail(Status.NoDetour)
        return append(bx, by, bz, tB, bFrame, profile) and tB or nil
    end
    local frame = detourFrame(i)
    local keptX, keptY, keptZ = wx, wy, wz
    local tx, ty, tz = bx, by, bz
    if endRule then tx, ty, tz = legEnd(endRule, wx, wy, wz) end
    local push = predictedPush(profile, i, ax, ay, az, wx, wy, wz, tx, ty, tz, run)
    if push then
        local px, py, pz = detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tB, flip, push)
        if px then keptX, keptY, keptZ = px, py, pz end
    end
    local kept, tEnd = planDetourThrough(ax, ay, az, t0, keptX, keptY, keptZ, frame, bx, by, bz, bFrame, depth, run, endRule)
    if kept then return tEnd end
    local tW = planLeg(ax, ay, az, t0, wx, wy, wz, frame, depth + 1, headRun(run))
    if not tW then return nil end
    return planLeg(wx, wy, wz, tW, bx, by, bz, bFrame, depth + 1, tailRun(run), endRule)
end

---Plans the leg a to b around obstacle `i` on one side, and on the other when that side's legs
---fail. When both fail, a body's first side stands, and a zone is given up: nothing is kept and
---`planned` is false, so the leg crosses the zone.
---@return number|nil tEnd The time at b, nil when the plan stopped
---@return boolean planned False for a zone that cannot be gone around
local function planAround(profile, i, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0, tIn, tB, depth, run, endRule)
    local savedWritten, savedFailures, savedStatus = written, failures, status
    local function detour(flip)
        return planDetour(profile, i, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0, tIn, tB, depth, flip, run,
            endRule)
    end
    local tEnd = detour(false)
    if tEnd == nil or failures == savedFailures then return tEnd, true end

    written, failures, status = savedWritten, savedFailures, savedStatus
    tEnd = detour(true)
    if tEnd == nil or failures == savedFailures then return tEnd, true end

    written, failures, status = savedWritten, savedFailures, savedStatus
    if i >= bodyCount then return nil, false end
    return detour(false), true
end

---Plans the leg from a to b (a point in frame `bFrame`), starting at time t0, and writes its
---waypoints. The leg's timeline is the drive's where that is sooner. A detour that fails
---further down (a waypoint inside another body, a leg that cannot be cleared) is redone on the
---other side of the body; if both fail, the first side's result stands. `run` (nil for none) is
---the drive run the leg belongs to: how far it goes on past b, and what it starts from. A leg with
---an end rule (`endRule`, `END.goal` or `END.entry`) ends where the rule puts it from its own start
---(`legEnd`), not at b, and so does the tail of a detour on it.
---@return number|nil tEnd The time at the leg's end, nil when the plan stopped
function planLeg(ax, ay, az, t0, bx, by, bz, bFrame, depth, run, endRule)
    if endRule then bx, by, bz = legEnd(endRule, ax, ay, az) end
    local dx, dy, dz = bx - ax, by - ay, bz - az
    local length = sqrt(dx * dx + dy * dy + dz * dz)
    if length <= MIN_LENGTH then
        local tB = t0 + length / speed
        return append(bx, by, bz, tB, bFrame, nil) and tB or nil
    end
    local ux, uy, uz = dx / length, dy / length, dz / length
    -- The leg's profile lives at its depth: the legs of a detour use the next depth, so a retry on
    -- the other side finds this one intact.
    local profile = legProfile(depth)
    buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run)
    local tB = profile.t[profile.count]
    local waived = legWaived(depth)
    while true do
        local obstacle, tIn = firstHit(profile, ax, ay, az, ux, uy, uz, t0, tB, obstacleCount, waived)
        local isZone = obstacle >= bodyCount
        if obstacle < 0 or (depth >= maxDepth and not isZone) then
            if obstacle >= 0 then fail(Status.DepthExceeded) end
            return append(bx, by, bz, tB, bFrame, profile) and tB or nil
        end
        -- A zone is crossed at the depth limit, when going round it costs more, and when no detour
        -- clears it; the leg then goes on to its next hit.
        local goAround = not isZone
            or (depth < maxDepth
                and zoneWorthDetour(profile, obstacle, ax, ay, az, bx, by, bz, length, ux, uy, uz, tIn, tB))
        if goAround then
            local tEnd, planned = planAround(profile, obstacle, ax, ay, az, bx, by, bz, bFrame, length, ux, uy, uz, t0,
                tIn, tB, depth, run, endRule)
            if planned then return tEnd end
        end
        waived[obstacle] = true
    end
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
            local hit = firstHit(chordProfile, px, py, pz, dx / length, dy / length, dz / length, t, t + length / speed,
                bodyCount, nil)
            if hit >= 0 then return false end
        end
        px, py, pz, t = qx, qy, qz, t + duration
    end
    return true
end

---The distance before a corner at which a ship under the drive turns its nose to the new leg, so
---that its velocity, which follows the nose with the drag's time constant, meets that leg: the
---distance it covers in one time constant.
---@param speed number Speed at the corner, game units per second
---@param drag number Linear drag rate, per second, above zero
---@return number Game units
function SystemPlanner.driveTurnLead(speed, drag)
    return speed / drag
end

---The point of the path of a ship that snaps its nose to the new leg when it is `lead` before the
---corner `c`, with `u1` the unit direction of the leg before and `u2` that of the leg after:
---`c + lead · (u2 · (τ − 1 + e^-τ) − u1 · e^-τ)`, with `τ` the time since the turn began in the
---drag's time constants. At `τ = 0` it is `lead` before the corner, at `τ = 1` at the corner (to
---first order), and it tends to the new leg. The one place this model is written.
function SystemPlanner.pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, tau)
    local decay = exp(-tau)
    local along = lead * (tau - 1 + decay)
    local back = lead * decay
    return cx + u2x * along - u1x * back, cy + u2y * along - u1y * back, cz + u2z * along - u1z * back
end

---The time, in the drag's time constants from the start of the turn (a lead before the corner), at
---which the velocity of a drive turn of `turn` radians is within `DRIVE_TURN_TOLERANCE` of the new
---heading. At least the lead: the turn lasts to the corner.
---@param turn number Radians, above zero
---@return number
local function driveTurnSettle(turn)
    return max(DRIVE_TURN_LEAD_TAUS, log(2 * sin(turn * HALF) / DRIVE_TURN_TOLERANCE))
end

---The first body the drive turn of a corner hits, as `DRIVE_TURN_CHORDS` chords flown over the
---turn's time at the run's speed, or -1 when it clears every body. The turn begins one time
---constant before the corner and lasts until it has settled.
---@return integer body Body index (0-based), or -1
local function driveTurnBlocker(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, tCorner, turn)
    local step = driveTurnSettle(turn) / DRIVE_TURN_CHORDS
    local duration = step / shipDrag
    local px, py, pz = SystemPlanner.pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, 0)
    local t = tCorner - DRIVE_TURN_LEAD_TAUS / shipDrag
    local times, along, vmax = chordProfile.t, chordProfile.s, chordProfile.vmax
    for j = 1, DRIVE_TURN_CHORDS do
        local qx, qy, qz = SystemPlanner.pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, step * j)
        local dx, dy, dz = qx - px, qy - py, qz - pz
        local length = sqrt(dx * dx + dy * dy + dz * dz)
        if length > MIN_LENGTH then
            times[1], along[1], vmax[1] = t, 0, length / duration
            times[2], along[2] = t + duration, length
            chordProfile.count, chordProfile.drive, chordProfile.through = 2, false, false
            local hit = firstHit(chordProfile, px, py, pz, dx / length, dy / length, dz / length, t, t + duration, bodyCount, nil)
            if hit >= 0 then return hit end
        end
        px, py, pz, t = qx, qy, qz, t + duration
    end
    return -1
end

---Whether a drive run keeps going through the corner `c` between the points before it (`p`) and
---after it (`n`), reached at `tCorner` at the run's speed `v` there. All of these hold: the turn
---does not slow the ship below the hand-over (`v · cos(turn / 2)`, the lowest speed of the turn, is
---at least `HANDOVER_MULT` times the planning speed); the turn fits in the legs (the lead before the
---corner, and the settling after it, which grows with the turn, take at most `ARC_LEG_SHARE` of
---each leg); and the path of the
---turn clears the bodies. A corner that barely turns holds.
---A corner that passes the first two and fails the third also returns the body its turn hits and
---the turn's cut: how far the turn's path passes the corner on its inside (the distance of the
---model's point one time constant into the turn from the corner).
---@return boolean holds
---@return integer|nil blocker Body index (0-based), only when the turn's path hits a body
---@return number|nil cut Metres, only with `blocker`
function driveCornerHolds(px, py, pz, cx, cy, cz, nx, ny, nz, v, tCorner)
    local turn, la, lb, u1x, u1y, u1z, u2x, u2y, u2z = cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    if not turn then return false end
    if turn < MIN_ARC_ANGLE then return true end
    if v * cos(turn * HALF) < TravelDriveMath.HANDOVER_MULT * speed then return false end
    local lead = SystemPlanner.driveTurnLead(v, shipDrag)
    local settle = lead * (driveTurnSettle(turn) - DRIVE_TURN_LEAD_TAUS)
    if lead > ARC_LEG_SHARE * la or settle > ARC_LEG_SHARE * lb then return false end
    local blocker = driveTurnBlocker(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, tCorner, turn)
    if blocker < 0 then return true end
    local qx, qy, qz = SystemPlanner.pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, DRIVE_TURN_LEAD_TAUS)
    return false, blocker, sqrt((qx - cx) ^ 2 + (qy - cy) ^ 2 + (qz - cz) ^ 2)
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

---The point of waypoint `k` of the output (the start for -1) in the legs' frame: its offset from
---its frame body's position at the waypoint's time, as seen from the legs' frame.
function legWaypointPoint(k, sx, sy, sz)
    if k < 0 then
        if legFrame == SYSTEM_FRAME then return sx, sy, sz end
        local fx, fy, fz = bodyPosition(legFrame, 0)
        return sx - fx, sy - fy, sz - fz
    end
    local base = k * OUT_STRIDE
    local frame, x, y, z = out[base + OUT_FRAME], out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z]
    if frame == SYSTEM_FRAME then return x, y, z end
    local cx, cy, cz = legBodyPosition(frame, out[base + OUT_T])
    return x + cx, y + cy, z + cz
end

---Gives every corner of the written plan its arc radius and passing speed. A corner is rounded in
---the frame its two legs are flown in; the corner where the frame changes (the entry into a well)
---is not rounded, and is passed at the run's speed when a drive run goes on through it, else at
---the planning speed. A corner the drive run goes through keeps radius zero and the run's speed:
---it was judged on the legs it has now (`planDetourThrough`). The goal keeps radius and speed
---zero, since the ship stops there. The plan's times stay as the legs gave them.
local function roundCorners(sx, sy, sz)
    for k = 0, written - 2 do
        local base = k * OUT_STRIDE
        local inFrame = legFrameOf[k + 1]
        local through = out[base + OUT_THROUGH] ~= 0
        if inFrame ~= legFrameOf[k + 2] then
            out[base + OUT_RADIUS], out[base + OUT_SPEED] = 0, through and out[base + OUT_SPEED] or speed
        else
            if through then
                out[base + OUT_RADIUS] = 0
            else
                setLegFrame(inFrame)
                local px, py, pz = legWaypointPoint(k - 1, sx, sy, sz)
                local cx, cy, cz = legWaypointPoint(k, sx, sy, sz)
                local nx, ny, nz = legWaypointPoint(k + 1, sx, sy, sz)
                out[base + OUT_RADIUS], out[base + OUT_SPEED] =
                    roundCorner(px, py, pz, cx, cy, cz, nx, ny, nz, out[base + OUT_T])
            end
        end
    end
end

---Where the goal is at `time`: the point (gx, gy, gz) itself in the system frame, else that offset
---from the goal's frame body.
local function goalPoint(gx, gy, gz, time)
    if goalFrame == SYSTEM_FRAME then return gx, gy, gz end
    local cx, cy, cz = bodyPosition(goalFrame, time)
    return gx + cx, gy + cy, gz + cz
end

---How the goal, an offset (gx, gy, gz) from its body, relates to the body's well: the well applies
---when the offset is inside the entry sphere, which is larger than the body's safe sphere (else
---the entry would be inside it). Sets `entryRadius`.
---@return integer mode `WELL_NONE`, `WELL_INSIDE` (the start is inside the well) or `WELL_ENTRY`
local function chooseWellMode(sx, sy, sz, gx, gy, gz)
    entryRadius = 0
    if goalFrame == SYSTEM_FRAME then return WELL_NONE end
    local base = goalFrame * BODY_STRIDE
    local wellRadius = bodies[base + ZONE_RADIUS]
    local entry = wellRadius * (1 - WELL_ENTRY_MARGIN)
    if wellRadius <= 0 or entry <= bodies[base + SAFE_RADIUS] then return WELL_NONE end
    -- The goal's stop point is at most `goalCut.standOff` from the goal's offset.
    if sqrt(gx * gx + gy * gy + gz * gz) + goalCut.standOff >= entry then return WELL_NONE end
    entryRadius = entry
    local cx, cy, cz = bodyPosition(goalFrame, 0)
    local dx, dy, dz = sx - cx, sy - cy, sz - cz
    return sqrt(dx * dx + dy * dy + dz * dz) < wellRadius and WELL_INSIDE or WELL_ENTRY
end

---Plans the system part to the entry and the part in the body's frame from there to the goal. The
---system part's last leg ends at the entry on its own line to the goal (`legEnd`), so the entry,
---wherever a detour leaves the leg, is on the line from the leg's start through the goal. With a
---`run` that leg goes on through the entry, and the part in the frame is one leg that continues it.
---@return number|nil tEnd The time the plan reaches its last point, nil when it stopped
---@return number|nil tEntry The time at the entry
local function planEntryLegs(sx, sy, sz, gx, gy, gz, run)
    setLegFrame(SYSTEM_FRAME)
    -- The entry is not a point given to the leg: the leg works it out from its own start.
    local tEntry = planLeg(sx, sy, sz, 0, 0, 0, 0, goalFrame, 0, run, END.entry)
    if not tEntry then return nil end
    entryIndex = written - 1
    local carried = nil
    if out[entryIndex * OUT_STRIDE + OUT_THROUGH] ~= 0 then
        carried = { beyond = 0, m = runEndMultiplier, v = runEndSpeed }
    end
    -- The entry's offset, from the start of the leg that wrote it, as `legEnd` worked it out.
    local px, py, pz = legWaypointPoint(entryIndex - 1, sx, sy, sz)
    local ex, ey, ez = entryOffset(px, py, pz, gx, gy, gz, entryCut.time)
    entryCut.offsetX, entryCut.offsetY, entryCut.offsetZ = ex, ey, ez
    setLegFrame(goalFrame)
    return planLeg(ex, ey, ez, tEntry, gx, gy, gz, goalFrame, 0, carried, END.goal), tEntry
end

---Whether the drive run goes on through the entry waypoint written last.
local function entryIsThrough()
    return out[entryIndex * OUT_STRIDE + OUT_THROUGH] ~= 0
end

---Whether the part in the body's frame is the continuation of a run: one straight leg from the
---entry to the goal, flown under the drive. Else the run ends at the entry.
local function runReachesGoal()
    local goalBase = (written - 1) * OUT_STRIDE
    return written == entryIndex + 2 and out[goalBase + OUT_DRIVE] ~= 0
end

---Plans the part of the goal's well: the whole plan in the body's frame when the start is inside
---the well, else the system part to the entry and the part in the body's frame from there. Notes
---the entry's waypoint and offset. Under the drive, the system part's last leg is planned to go on
---through the entry when the part in the frame is long enough to be driven; if that part turns out
---not to be one driven leg (a detour, or too many timeline steps), the plan is made again with the
---run ending at the entry.
---@return number|nil tEnd The time the plan reaches its last point, nil when it stopped
---@return number|nil tEntry The time at the entry, nil for a ship that starts inside the well
local function planWell(sx, sy, sz, gx, gy, gz, entryTime)
    -- The legs in the body's frame are offsets from the body, and so is the goal.
    goalCut.x, goalCut.y, goalCut.z = gx, gy, gz
    if wellMode == WELL_INSIDE then
        setLegFrame(goalFrame)
        local fx, fy, fz = bodyPosition(goalFrame, 0)
        return planLeg(sx - fx, sy - fy, sz - fz, 0, gx, gy, gz, goalFrame, 0, nil, END.goal)
    end
    entryCut.x, entryCut.y, entryCut.z, entryCut.time = gx, gy, gz, entryTime
    -- The entry on the line from the start: the plan's entry when no detour comes before it, and
    -- otherwise the estimate of its distance from the start and of the part in the frame's length.
    local ex, ey, ez = entryOffset(sx, sy, sz, gx, gy, gz, entryTime)
    local cx, cy, cz = bodyPosition(goalFrame, entryTime)
    entryChord = sqrt((ex + cx - sx) ^ 2 + (ey + cy - sy) ^ 2 + (ez + cz - sz) ^ 2)
    local stopX, stopY, stopZ = goalStop(ex, ey, ez)
    local framedLength = sqrt((stopX - ex) ^ 2 + (stopY - ey) ^ 2 + (stopZ - ez) ^ 2)
    if driveOn and framedLength > driveRelease then
        local tEnd, tEntry = planEntryLegs(sx, sy, sz, gx, gy, gz, { beyond = framedLength })
        -- A stopped plan, a plan whose last system leg is not a run, and a run that reaches the goal
        -- stand as they are.
        if tEnd == nil or not entryIsThrough() or runReachesGoal() then return tEnd, tEntry end
        written, status, failures = 0, Status.Ok, 0
    end
    return planEntryLegs(sx, sy, sz, gx, gy, gz, nil)
end

---Moves the goal's time, the last waypoint written, from the time the ship reaches the goal point
---(`tEnd`) to the time it comes within `arrivalRadius` of it along the last leg's timeline: the
---leg's start for a leg shorter than the radius. The path itself is not changed.
---@param tEnd number|nil The time the plan reaches its last point, nil when it stopped
---@return number|nil tArrive The goal's time
local function arrivalTime(tEnd)
    local profile = lastProfile
    if not tEnd or arrivalRadius <= 0 or not profile then return tEnd end
    local tArrive = timeAtDistance(profile, profile.s[profile.count] - arrivalRadius)
    out[(written - 1) * OUT_STRIDE + OUT_T] = tArrive
    return tArrive
end

---One full plan toward the goal as it is at `goalTime` (and, for a goal in a well that the ship
---has to enter, the entry as it is at `entryTime`).
---@return boolean ok False when an endpoint is inside a body's hull
---@return number|nil tEnd The time the ship arrives at the goal, nil when the plan stopped
---@return number|nil tEntry The time at the entry, when the plan has one
local function planPass(sx, sy, sz, gx, gy, gz, goalTime, entryTime)
    written, status, failures, entryIndex, lastProfile = 0, Status.Ok, 0, -1, nil
    resetHitRadii()
    setLegFrame(SYSTEM_FRAME)
    local px, py, pz = goalPoint(gx, gy, gz, goalTime)
    goalCut.x, goalCut.y, goalCut.z = px, py, pz
    -- The endpoint checks are made at the stop point of the straight line from the start: the goal
    -- point is the target's centre, inside its hull.
    local qx, qy, qz = goalStop(sx, sy, sz)
    if not (shrinkAround(sx, sy, sz, 0) and shrinkAround(qx, qy, qz, goalTime)) then return false end
    markIgnoredZones(sx, sy, sz, qx, qy, qz, goalTime)
    if wellMode == WELL_NONE then
        return true, arrivalTime(planLeg(sx, sy, sz, 0, px, py, pz, goalFrame, 0, nil, END.goal))
    end
    local tEnd, tEntry = planWell(sx, sy, sz, gx, gy, gz, entryTime)
    return true, arrivalTime(tEnd), tEntry
end

---Whether the numbers and the frame index of a request are usable.
local function validRequest(sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit, outCapacity)
    local finite = isFinite(sx) and isFinite(sy) and isFinite(sz) and isFinite(gx) and isFinite(gy) and isFinite(gz)
    local frameValid = frame == math.floor(frame) and frame >= SYSTEM_FRAME and frame < bodyCount
    local shipValid = accel > 0 and isFinite(accel) and drag >= 0 and isFinite(drag)
    local arrivalValid = isFinite(arrivalRadius) and arrivalRadius >= 0 and isFinite(goalCut.standOff) and goalCut.standOff >= 0
    return finite and frameValid and shipValid and arrivalValid and planSpeed > 0 and isFinite(planSpeed)
        and depthLimit >= 0 and outCapacity >= 1 and bodyCount >= 0
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
    local minLength = drive[DRIVE_MIN_LENGTH]
    local valid = isFinite(openCap) and openCap >= 1 and isFinite(charge) and charge >= 0
        and isFinite(rampUp) and rampUp > 0 and isFinite(rampDown) and rampDown > 0 and shipDrag > 0
        and isFinite(minLength) and minLength >= 0
    if not valid then
        driveOn = false
        return false
    end
    driveOpenCap, driveCharge, driveRampUp, driveRampDown = openCap, charge, rampUp, rampDown
    driveMinLength = minLength
    driveShortestLag = min(1 / rampUp, 1 / rampDown, 1 / shipDrag)
    driveBrakingTime = TravelDriveMath.brakingTime(rampDown, shipDrag)
    driveRelease = TravelDriveMath.HANDOVER_MULT * driveBrakingTime * speed
    return true
end

---Whether the goal's body can move as fast as the ship can fly before the ship reaches the body's
---well, so that the ship may never catch the goal. A ship that starts inside the well is carried
---with the body, and the body's speed does not matter. Without a well to enter, the ship's top
---speed is the planning speed; with one, it is the drive's top speed when the plan has a drive.
local function outrunsBeforePlanning(frame, planSpeed)
    if frame == SYSTEM_FRAME or wellMode == WELL_INSIDE then return false end
    local top = planSpeed
    if wellMode == WELL_ENTRY and driveOn then top = planSpeed * driveOpenCap end
    return speedBound[frame + 1] >= top
end

---Plans toward the goal until the goal's time settles, then rounds the corners.
---@return integer status
---@return integer waypointCount
local function solve(sx, sy, sz, frame, gx, gy, gz, planSpeed)
    local fx, fy, fz = goalPoint(gx, gy, gz, 0)
    local dx, dy, dz = fx - sx, fy - sy, fz - sz
    local goalTime = max(0, sqrt(dx * dx + dy * dy + dz * dz) - goalCut.standOff) / planSpeed
    local entryTime = goalTime
    local settled, tEntry = false, nil
    for _ = 1, frame == SYSTEM_FRAME and 1 or MAX_GOAL_ITERATIONS do
        local ok, tEnd
        ok, tEnd, tEntry = planPass(sx, sy, sz, gx, gy, gz, goalTime, entryTime)
        if not ok then return Status.EndpointInsideBody, 0 end
        settled = frame == SYSTEM_FRAME or tEnd == nil
            or (abs(tEnd - goalTime) <= GOAL_TIME_TOLERANCE
                and (tEntry == nil or abs(tEntry - entryTime) <= GOAL_TIME_TOLERANCE))
        if settled then break end
        goalTime, entryTime = tEnd, tEntry or entryTime
    end
    if not settled then
        -- A ship that cannot close on the well's entry never settles on it.
        if wellMode == WELL_ENTRY and entryChord <= speedBound[frame + 1] * tEntry then return Status.GoalOutrun, 0 end
        if status == Status.Ok then status = Status.GoalNotSettled end
    end

    -- The goal and the entry keep their offsets exactly, not the ones rounded from the last pass's
    -- points. A goal with a stand-off is the stop point of its last leg, which the leg wrote.
    if frame ~= SYSTEM_FRAME and written > 0 and status ~= Status.CapacityExceeded then
        local base = (written - 1) * OUT_STRIDE
        if goalCut.standOff <= 0 then
            out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z] = gx, gy, gz
        end
        if entryIndex >= 0 then
            base = entryIndex * OUT_STRIDE
            out[base + OUT_X], out[base + OUT_Y], out[base + OUT_Z] =
                entryCut.offsetX, entryCut.offsetY, entryCut.offsetZ
        end
    end
    if status ~= Status.CapacityExceeded then roundCorners(sx, sy, sz) end
    return status, written
end

---Plans a path from the start to the goal through the bodies. The goal is the point (gx, gy, gz)
---in the system frame, or that offset (world axes) from the centre of body `frame`; a goal on a
---moving body is planned for the time the ship gets there, and a goal inside that body's well
---(`ZoneRadius`) is planned to the well's edge in the system, then in the body's frame, which
---carries the ship. With a drive block, a leg is flown under the travel drive where that is sooner
---and the leg is at least the drive's minimum length, and `Out.Drive` marks it; `planSpeed` is then
---the ship's speed without the drive. A drive run goes on through the well's entry to the goal,
---and `Out.Through` marks the entry then. The goal's time is when the ship comes within
---`arrivalRadius` of the goal; the path and the other times are those of a plan without a radius.
---A goal with a `standOff` is a centre: a leg that ends at it stops `standOff` short of it, on the
---line from the leg's start, so the last leg points at the centre; the last waypoint is that stop point.
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
---@param arrival number|nil The arrival radius, game units, zero or more, nil for zero: how close to the goal the ship counts as arrived
---@param zonesBuffer userdata|nil `double[?]`, `zoneCount * ZONE_STRIDE` values: the cost zones, nil for none
---@param zoneCountArg integer|nil Number of cost zones, zero or more, nil for zero
---@param standOff number|nil The distance the last leg stops short of the goal, game units, zero or more, nil for zero
---@return integer status A `Status` value
---@return integer waypointCount Waypoints written, the goal last when the status allows a plan
function SystemPlanner.plan(bodiesBuffer, count, sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit,
                            outBuffer, outCapacity, driveBuffer, arrival, zonesBuffer, zoneCountArg, standOff)
    bodies, bodyCount, speed, maxDepth, out, capacity = bodiesBuffer, count, planSpeed, depthLimit, outBuffer, outCapacity
    goalFrame, written, status, failures = frame, 0, Status.Ok, 0
    arrivalRadius = arrival or 0
    goalCut.standOff = standOff or 0
    zones, zoneCount = zonesBuffer, zoneCountArg or 0
    if not validRequest(sx, sy, sz, frame, gx, gy, gz, planSpeed, accel, drag, depthLimit, outCapacity) then
        return Status.InvalidInput, 0
    end
    local zonesValid = isFinite(zoneCount) and zoneCount >= 0 and zoneCount == math.floor(zoneCount)
    if not (zonesValid and (zoneCount == 0 or zones)) then
        return Status.InvalidInput, 0
    end
    obstacleCount = bodyCount + zoneCount
    setShip(accel, drag)
    if not setDrive(driveBuffer) or not prepareBodies() or not prepareZones() then return Status.InvalidInput, 0 end
    wellMode = chooseWellMode(sx, sy, sz, gx, gy, gz)
    if outrunsBeforePlanning(frame, planSpeed) then return Status.GoalOutrun, 0 end

    return solve(sx, sy, sz, frame, gx, gy, gz, planSpeed)
end

return SystemPlanner
