--- System planner kernel: a path through a system of orbiting bodies as waypoints (a body index and
--- an offset from it). Straight legs get detours around the bodies' safe spheres at their predicted
--- positions, corners are rounded into arcs, legs are timed under the travel drive where that is
--- sooner (a gentle detour corner keeps the drive run), a goal inside its body's well is flown in the body's frame from the well's edge, and a
--- cost zone is gone around where that is cheaper than crossing it.
--- Pure numbers in and out (no entities, no globals), so it runs on a worker or in a test state.
--- Not reentrant: it keeps module scratch state. This is part 6 of 6, the plan itself and the public
--- surface; `SystemPlannerLayout`, `Profile`, `Scene`, `Timeline` and `Corners` are the other parts.
--- Design: doc/script/nav/system-planner.md.

local Layout   = require("Shared.Helpers.SystemPlannerLayout")
local Profile  = require("Shared.Helpers.SystemPlannerProfile")
local Scene    = require("Shared.Helpers.SystemPlannerScene")
local Timeline = require("Shared.Helpers.SystemPlannerTimeline")
local Corners  = require("Shared.Helpers.SystemPlannerCorners")

local sqrt = math.sqrt
local max, min, abs = math.max, math.min, math.abs

local SystemPlanner = {}

local Status = Layout.Status
local BODY_STRIDE, ZONE_RADIUS, SAFE_RADIUS = Layout.BODY_STRIDE, Layout.ZONE_RADIUS, Layout.SAFE_RADIUS
local ZONE_COST, ZONE_STRIDE = Layout.ZONE_COST, Layout.ZONE_STRIDE
local OUT_FRAME, OUT_X, OUT_Y, OUT_Z, OUT_T = Layout.OUT_FRAME, Layout.OUT_X, Layout.OUT_Y, Layout.OUT_Z, Layout.OUT_T
local OUT_RADIUS, OUT_SPEED, OUT_DRIVE, OUT_THROUGH, OUT_STRIDE =
    Layout.OUT_RADIUS, Layout.OUT_SPEED, Layout.OUT_DRIVE, Layout.OUT_THROUGH, Layout.OUT_STRIDE
local SYSTEM_FRAME, MIN_LENGTH, isFinite = Layout.SYSTEM_FRAME, Layout.MIN_LENGTH, Layout.isFinite

SystemPlanner.Status = Status
SystemPlanner.BODY_STRIDE = BODY_STRIDE
SystemPlanner.Body = Layout.Body
SystemPlanner.OUT_STRIDE = OUT_STRIDE
SystemPlanner.Out = Layout.Out
SystemPlanner.DRIVE_STRIDE = Layout.DRIVE_STRIDE
SystemPlanner.Drive = Layout.Drive
SystemPlanner.ZONE_STRIDE = ZONE_STRIDE
SystemPlanner.Zone = Layout.Zone
SystemPlanner.SYSTEM_FRAME = SYSTEM_FRAME
SystemPlanner.TURN_THRUST_SHARE = Corners.TURN_THRUST_SHARE
SystemPlanner.turnSpeed = Corners.turnSpeed
SystemPlanner.cornerArc = Corners.cornerArc
SystemPlanner.pointOnArc = Corners.pointOnArc
SystemPlanner.driveTurnLead = Corners.driveTurnLead
SystemPlanner.pointOnDriveTurn = Corners.pointOnDriveTurn

local bodyPosition, legBodyPosition, detourFrame = Scene.bodyPosition, Scene.legBodyPosition, Scene.detourFrame
local firstHit, overlapExitTime, detourPoint = Scene.firstHit, Scene.overlapExitTime, Scene.detourPoint
local legWaived, speedBound = Scene.legWaived, Scene.speedBound
local isDriveOn, driveOpenCap, driveMinLength, driveRelease =
    Timeline.isOn, Timeline.openCap, Timeline.minLength, Timeline.release
local buildProfile = Timeline.buildProfile
local driveCornerHolds, roundCorner = Corners.driveCornerHolds, Corners.roundCorner
local legProfile, distanceAt, locate = Profile.forDepth, Profile.distanceAt, Profile.locate
local timeAtDistance, speedAtDistance = Profile.timeAtDistance, Profile.speedAtDistance

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

SystemPlanner.WELL_ENTRY_MARGIN = WELL_ENTRY_MARGIN

-- Scratch state of the running plan.
local bodies, bodyCount, speed, maxDepth, out, capacity, written, status, failures, goalFrame
-- The cost zones: their buffer and count. Obstacles are numbered bodies first (0 .. bodyCount - 1),
-- then zones (bodyCount .. obstacleCount - 1), so the hit test and the detour work on both.
local zones, zoneCount, obstacleCount = nil, 0, 0
-- The frame the legs being planned are flown in (see `Scene.setLegFrame`), and per waypoint written
-- the frame of the leg that ends there.
local legFrame, legFrameOf = SYSTEM_FRAME, {}
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

---Flies the legs that follow in the frame of body `frame`, or in the system for SYSTEM_FRAME.
local function setLegFrame(frame)
    legFrame = frame
    Scene.setLegFrame(frame)
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

---Where the straight line from `a` to `g` first crosses the sphere of `radius` about the origin:
---`a + λ(g − a)`, with `λ` the smaller root of `|a + λ(g − a)|² = radius²`. A line that misses the
---sphere meets it at its closest approach, and an `a` already inside it enters at once (`λ = 0`).
---Both points are offsets from the sphere's centre.
---@return number x
---@return number y
---@return number z
function SystemPlanner.sphereEntry(ax, ay, az, gx, gy, gz, radius)
    local dx, dy, dz = gx - ax, gy - ay, gz - az
    local qa = dx * dx + dy * dy + dz * dz
    local qb = ax * dx + ay * dy + az * dz
    local qc = ax * ax + ay * ay + az * az - radius * radius
    local lambda = 0
    if qc > 0 and qa > MIN_LENGTH then
        lambda = max(0, (-qb - sqrt(max(0, qb * qb - qa * qc))) / qa)
    end
    return ax + dx * lambda, ay + dy * lambda, az + dz * lambda
end

---The offset from the goal's body, at `entryTime`, of the point where the straight line from the
---start to the goal first crosses the entry sphere (`SystemPlanner.sphereEntry`).
local function entryOffset(sx, sy, sz, gx, gy, gz, entryTime)
    local cx, cy, cz = bodyPosition(goalFrame, entryTime)
    return SystemPlanner.sphereEntry(sx - cx, sy - cy, sz - cz, gx, gy, gz, entryRadius)
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

local legWaypointPoint

---Whether a detour a to w to b may keep a drive run through the corner w: the drive is planned, a
---run starts at or goes on through the detour's head, and a run is left after the corner (`rest`
---long, or the run goes on past b by `before`).
---@return boolean
---@return number rest The distance from w to b
---@return number before The length the run goes on past b
local function detourMayKeepRun(ax, ay, az, wx, wy, wz, bx, by, bz, run)
    local rest = sqrt((bx - wx) ^ 2 + (by - wy) ^ 2 + (bz - wz) ^ 2)
    local before = run and run.beyond or 0
    local startsRun = run and run.m or sqrt((wx - ax) ^ 2 + (wy - ay) ^ 2 + (wz - az) ^ 2) >= driveMinLength()
    return isDriveOn() and startsRun and (before > 0 or rest > driveRelease()), rest, before
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
---`Timeline.buildProfile`). The drive run goes through the detour's corner when that holds on the
---plan of its tail (`planDetourThrough`), else the corner ends it at the detour's own waypoint. A
---corner expected to fail only because its turn cuts into the obstacle (`predictedPush`) is tried
---with the waypoint pushed out by the turn's cut.
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
    if buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run) and status == Status.Ok then
        status = Status.DriveNotTimed
    end
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
    if isDriveOn() and framedLength > driveRelease() then
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
    Scene.resetHitRadii()
    setLegFrame(SYSTEM_FRAME)
    local px, py, pz = goalPoint(gx, gy, gz, goalTime)
    goalCut.x, goalCut.y, goalCut.z = px, py, pz
    -- The endpoint checks are made at the stop point of the straight line from the start: the goal
    -- point is the target's centre, inside its hull.
    local qx, qy, qz = goalStop(sx, sy, sz)
    if not (Scene.shrinkAround(sx, sy, sz, 0) and Scene.shrinkAround(qx, qy, qz, goalTime)) then return false end
    Scene.markIgnoredZones(sx, sy, sz, qx, qy, qz, goalTime)
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

---Whether the goal's body can move as fast as the ship can fly before the ship reaches the body's
---well, so that the ship may never catch the goal. A ship that starts inside the well is carried
---with the body, and the body's speed does not matter. Without a well to enter, the ship's top
---speed is the planning speed; with one, it is the drive's top speed when the plan has a drive.
local function outrunsBeforePlanning(frame, planSpeed)
    if frame == SYSTEM_FRAME or wellMode == WELL_INSIDE then return false end
    local top = planSpeed
    if wellMode == WELL_ENTRY and isDriveOn() then top = planSpeed * driveOpenCap() end
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
    Scene.begin(bodies, bodyCount, zones, zoneCount, planSpeed)
    Corners.begin(planSpeed, bodyCount, accel, drag)
    if not Timeline.setDrive(driveBuffer, planSpeed, drag) or not Scene.prepare() then return Status.InvalidInput, 0 end
    wellMode = chooseWellMode(sx, sy, sz, gx, gy, gz)
    if outrunsBeforePlanning(frame, planSpeed) then return Status.GoalOutrun, 0 end

    return solve(sx, sy, sz, frame, gx, gy, gz, planSpeed)
end

return SystemPlanner
