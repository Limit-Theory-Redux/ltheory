-- System planner kernel, part 3 of 6: the bodies and cost zones of a plan as obstacles. Where they
-- are at a time, in the system or in a leg's frame; the hit search of a leg against their spheres;
-- and the detour waypoint around one. Not reentrant: it keeps module scratch state, set by `begin`.
-- Design: doc/script/nav/system-planner.md.

local OrbitMath       = require("Shared.Helpers.OrbitMath")
local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")
local Layout          = require("Shared.Helpers.SystemPlannerLayout")
local Profile         = require("Shared.Helpers.SystemPlannerProfile")

local sqrt, atan2, acos, sin, cos = math.sqrt, math.atan2, math.acos, math.sin, math.cos
local max, min, abs = math.max, math.min, math.abs

local BODY_STRIDE, PARENT, POS_X, POS_Y, POS_Z = Layout.BODY_STRIDE, Layout.PARENT, Layout.POS_X, Layout.POS_Y, Layout.POS_Z
local ORBIT_RADIUS, PHASE, ANGULAR_SPEED, INCLINATION = Layout.ORBIT_RADIUS, Layout.PHASE, Layout.ANGULAR_SPEED, Layout.INCLINATION
local BODY_RADIUS, SAFE_RADIUS, ZONE_RADIUS, ZONE_CAP = Layout.BODY_RADIUS, Layout.SAFE_RADIUS, Layout.ZONE_RADIUS, Layout.ZONE_CAP
local ZONE_FRAME, ZONE_X, ZONE_Y, ZONE_Z = Layout.ZONE_FRAME, Layout.ZONE_X, Layout.ZONE_Y, Layout.ZONE_Z
local ZONE_RADIUS_AT, ZONE_COST, ZONE_STRIDE = Layout.ZONE_RADIUS_AT, Layout.ZONE_COST, Layout.ZONE_STRIDE
local SYSTEM_FRAME, MIN_LENGTH, HALF, isFinite = Layout.SYSTEM_FRAME, Layout.MIN_LENGTH, Layout.HALF, Layout.isFinite
local locate, distanceAt = Profile.locate, Profile.distanceAt

local Scene = {}

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
-- The exit of a cost zone is found within a quarter radius by the overlap walk, then bisected this
-- many times: the length inside the zone is what the cross-or-go-around rule weighs, so it needs
-- more than the walk's step.
local ZONE_EXIT_BISECTIONS = 20
-- A drive leg is timed in steps. Each step covers at most this share of the distance to the
-- nearest feature that changes the drive's cap: a zone's edge or body, or the point where the
-- braking law starts to bind.
local DRIVE_FEATURE_STEP = 0.05
-- The distance to a zone's edge counts as at least this share of the zone's influence radius, so
-- a ship on the edge still takes steps.
local DRIVE_FEATURE_FLOOR = 0.01

Scene.DRIVE_FEATURE_STEP = DRIVE_FEATURE_STEP

-- Scratch state of the running plan: the bodies buffer and their count, the planning speed.
local bodies, bodyCount, speed
-- The cost zones: their buffer and count. Obstacles are numbered bodies first (0 .. bodyCount - 1),
-- then zones (bodyCount .. obstacleCount - 1), so the hit test and the detour work on both.
local zones, zoneCount = nil, 0
-- Per zone: whether this plan ignores it (it costs nothing, or it holds an endpoint). Per recursion
-- depth: the zones left out of the hit test of the leg being planned (ignored, or waived because
-- crossing is cheaper than going around).
local zoneIgnored, waivedByDepth = {}, {}
-- Per body: the speed bound of its orbit chain, and the radius of the sphere the hit test uses.
-- The tables are filled in place, so the other parts may hold them.
local speedBound, hitRadius = {}, {}
-- The frame the legs being planned are flown in: a body index (the legs are straight lines of
-- offsets from that body, which carries the ship), or SYSTEM_FRAME. Per body: the speed bound of
-- its motion relative to that frame.
local legFrame, legBound = SYSTEM_FRAME, {}

Scene.speedBound = speedBound

---Sets the bodies, the zones (nil buffer for none) and the planning speed of a plan.
function Scene.begin(bodiesBuffer, count, zonesBuffer, zoneCountArg, planSpeed)
    bodies, bodyCount, speed = bodiesBuffer, count, planSpeed
    zones, zoneCount = zonesBuffer, zoneCountArg
end

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

Scene.bodyPosition = bodyPosition

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

---Checks the bodies and the zones of the plan and fills the per-body scratch.
---@return boolean valid
function Scene.prepare()
    return prepareBodies() and prepareZones()
end

---Restores every body's hit radius to its safe radius, undoing the endpoint shrinks of a pass, and
---every zone's to its radius.
function Scene.resetHitRadii()
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
function Scene.shrinkAround(x, y, z, t)
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
function Scene.setLegFrame(frame)
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

Scene.legBodyPosition = legBodyPosition

---The frame a waypoint of a detour around obstacle `i` is stored in: the body itself, or a zone's
---frame. A zone in the system, passed inside a well's frame, is stored in that well's frame, where
---the leg's coordinates are offsets.
function Scene.detourFrame(i)
    if i < bodyCount then return i end
    local frame = zones[(i - bodyCount) * ZONE_STRIDE + ZONE_FRAME]
    if frame == SYSTEM_FRAME then return legFrame end
    return frame
end

---Sets which zones this plan ignores: one that costs nothing, and one that holds the start (at
---time zero) or the goal (at `goalTime`), since entering that one cannot be avoided.
function Scene.markIgnoredZones(sx, sy, sz, gx, gy, gz, goalTime)
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
function Scene.legWaived(depth)
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

---What the zones do to the drive at the point (x, y, z) at time `t`, for a ship moving at `v`: the
---multiplier they allow (the lowest of any zone's, at most the open-space cap `openCap`) and the
---longest step in seconds before the cap could have changed much. A step covers at most
---`DRIVE_FEATURE_STEP` of the distance to a zone's edge, counted relative to the zone's body and
---at its speed bound.
---@return number limit
---@return number step
function Scene.zoneState(openCap, x, y, z, t, v)
    local limit, step = openCap, math.huge
    for i = 0, bodyCount - 1 do
        local base = i * BODY_STRIDE
        local zoneRadius = bodies[base + ZONE_RADIUS]
        if zoneRadius > 0 then
            local cx, cy, cz = legBodyPosition(i, t)
            local dx, dy, dz = x - cx, y - cy, z - cz
            local d = sqrt(dx * dx + dy * dy + dz * dz)
            limit = min(limit, TravelDriveMath.zoneCap(openCap, bodies[base + ZONE_CAP], zoneRadius, d))
            local influence = zoneRadius * TravelDriveMath.INFLUENCE_RADIUS_FACTOR
            local gap = max(abs(d - influence), DRIVE_FEATURE_FLOOR * influence)
            step = min(step, DRIVE_FEATURE_STEP * gap / (v + legBound[i + 1]))
        end
    end
    return limit, step
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
function Scene.firstHit(profile, ax, ay, az, ux, uy, uz, t0, tEnd, limit, waived)
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
function Scene.overlapExitTime(profile, i, ax, ay, az, ux, uy, uz, tIn, tEnd)
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
function Scene.detourPoint(profile, i, ax, ay, az, length, ux, uy, uz, tIn, tEnd, flip, push)
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

return Scene
