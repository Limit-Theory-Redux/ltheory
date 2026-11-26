-- System planner kernel, part 5 of 6: the corners of a plan. The turn geometry (arcs, and the turn
-- of a ship under the drive, one model each that the planner and a ship that flies the plan share),
-- the check of a turn against the bodies, and the radius and speed a corner is passed at.
-- Not reentrant: it keeps module scratch state, set by `begin`.
-- Design: doc/script/nav/system-planner.md.

local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")
local Layout          = require("Shared.Helpers.SystemPlannerLayout")
local Profile         = require("Shared.Helpers.SystemPlannerProfile")
local Scene           = require("Shared.Helpers.SystemPlannerScene")

local sqrt, acos, sin, cos, tan, exp, log = math.sqrt, math.acos, math.sin, math.cos, math.tan, math.exp, math.log
local max, min = math.max, math.min

local MIN_LENGTH, HALF = Layout.MIN_LENGTH, Layout.HALF
local firstHit, flatProfile = Scene.firstHit, Profile.flat

local Corners = {}

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

-- A corner flown under the drive is a turn of the velocity toward the new leg with the drag's time
-- constant. The nose turns one time constant before the corner (the lead). The turn counts as done
-- when the velocity is off the new heading by at most this share of the run's speed (the error
-- starts at `2 · sin(turn / 2)` of the speed and falls by `e^-τ`), and it is checked against the
-- bodies as this many chords.
local DRIVE_TURN_LEAD_TAUS  = 1
local DRIVE_TURN_TOLERANCE  = 0.01
local DRIVE_TURN_CHORDS     = 12

Corners.TURN_THRUST_SHARE = TURN_THRUST_SHARE

-- The planning speed and the body count of the running plan.
local speed, bodyCount
-- The ship's forward acceleration and drag, its top speed (infinite without drag) and the arc
-- radius at the turn-speed knee (infinite without drag).
local shipAccel, shipDrag, shipTopSpeed, kneeRadius
-- The timeline of one chord of an arc or of a drive turn, checked against the bodies.
local chordProfile = Profile.new()

---Sets the plan's speed and body count and the ship scratch from its forward acceleration and drag:
---the top speed, and the arc radius at which the turn speed reaches `TURN_SPEED_KNEE` of its limit
---`share · accel / drag`, which is `s² / (share · accel · √(1 − knee²))` for `s = knee · limit`.
---Both are infinite without drag.
function Corners.begin(planSpeed, count, accel, drag)
    speed, bodyCount = planSpeed, count
    shipAccel, shipDrag = accel, drag
    if drag > 0 then
        shipTopSpeed = accel / drag
        kneeRadius = TURN_SPEED_KNEE * TURN_SPEED_KNEE * TURN_THRUST_SHARE * accel
            / (drag * drag * sqrt(1 - TURN_SPEED_KNEE * TURN_SPEED_KNEE))
    else
        shipTopSpeed, kneeRadius = math.huge, math.huge
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
function Corners.turnSpeed(accel, drag, radius, topSpeed)
    local thrust = TURN_THRUST_SHARE * accel
    if radius <= 0 or thrust <= 0 then return 0 end
    local dragSq = drag * drag
    local speedSq = HALF * radius * radius
        * (-dragSq + sqrt(dragSq * dragSq + 4 * thrust * thrust / (radius * radius)))
    return min(sqrt(speedSq), topSpeed)
end
local turnSpeed = Corners.turnSpeed

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
function Corners.cornerArc(px, py, pz, cx, cy, cz, nx, ny, nz, radius)
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
local cornerArc = Corners.cornerArc

---The point of an arc `angle` radians (zero to the arc's turn) from its entry, for the centre,
---the entry and the unit normal `cornerArc` returns.
function Corners.pointOnArc(ox, oy, oz, ex, ey, ez, kx, ky, kz, angle)
    local vx, vy, vz = ex - ox, ey - oy, ez - oz
    local c, s = cos(angle), sin(angle)
    local wx, wy, wz = ky * vz - kz * vy, kz * vx - kx * vz, kx * vy - ky * vx
    return ox + vx * c + wx * s, oy + vy * c + wy * s, oz + vz * c + wz * s
end
local pointOnArc = Corners.pointOnArc

---Whether the arc, from its entry at time `tEntry`, keeps clear of every body as a chain of
---chords flown at the planning speed. The chords lie on the centre's side of the arc, which is
---the side of the body the corner goes around, so a body there is found a little early; a body
---beyond the arc can be missed by up to the chords' sagitta.
local function arcIsClear(ex, ey, ez, ox, oy, oz, kx, ky, kz, turn, radius, tEntry)
    local px, py, pz, t = ex, ey, ez, tEntry
    local step = turn / ARC_CHORDS
    local duration = radius * step / speed
    for j = 1, ARC_CHORDS do
        local qx, qy, qz = pointOnArc(ox, oy, oz, ex, ey, ez, kx, ky, kz, step * j)
        local dx, dy, dz = qx - px, qy - py, qz - pz
        local length = sqrt(dx * dx + dy * dy + dz * dz)
        if length > MIN_LENGTH then
            flatProfile(chordProfile, t, length, speed)
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
function Corners.driveTurnLead(speed, drag)
    return speed / drag
end
local driveTurnLead = Corners.driveTurnLead

---The point of the path of a ship that snaps its nose to the new leg when it is `lead` before the
---corner `c`, with `u1` the unit direction of the leg before and `u2` that of the leg after:
---`c + lead · (u2 · (τ − 1 + e^-τ) − u1 · e^-τ)`, with `τ` the time since the turn began in the
---drag's time constants. At `τ = 0` it is `lead` before the corner, at `τ = 1` at the corner (to
---first order), and it tends to the new leg. The one place this model is written.
function Corners.pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, tau)
    local decay = exp(-tau)
    local along = lead * (tau - 1 + decay)
    local back = lead * decay
    return cx + u2x * along - u1x * back, cy + u2y * along - u1y * back, cz + u2z * along - u1z * back
end
local pointOnDriveTurn = Corners.pointOnDriveTurn

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
    local px, py, pz = pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, 0)
    local t = tCorner - DRIVE_TURN_LEAD_TAUS / shipDrag
    local times, along, vmax = chordProfile.t, chordProfile.s, chordProfile.vmax
    for j = 1, DRIVE_TURN_CHORDS do
        local qx, qy, qz = pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, step * j)
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
function Corners.driveCornerHolds(px, py, pz, cx, cy, cz, nx, ny, nz, v, tCorner)
    local turn, la, lb, u1x, u1y, u1z, u2x, u2y, u2z = cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    if not turn then return false end
    if turn < MIN_ARC_ANGLE then return true end
    if v * cos(turn * HALF) < TravelDriveMath.HANDOVER_MULT * speed then return false end
    local lead = driveTurnLead(v, shipDrag)
    local settle = lead * (driveTurnSettle(turn) - DRIVE_TURN_LEAD_TAUS)
    if lead > ARC_LEG_SHARE * la or settle > ARC_LEG_SHARE * lb then return false end
    local blocker = driveTurnBlocker(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, tCorner, turn)
    if blocker < 0 then return true end
    local qx, qy, qz = pointOnDriveTurn(cx, cy, cz, u1x, u1y, u1z, u2x, u2y, u2z, lead, DRIVE_TURN_LEAD_TAUS)
    return false, blocker, sqrt((qx - cx) ^ 2 + (qy - cy) ^ 2 + (qz - cz) ^ 2)
end

---The arc radius and passing speed of a corner reached at time `tCorner`. The radius is the
---largest the legs and the turn-speed knee allow that the bodies leave clear, halving until one
---does. A corner that barely turns is passed at the planning speed with no arc; one with no clear
---arc is left sharp, to be passed at rest.
---@return number radius Zero for no arc
---@return number speed
function Corners.roundCorner(px, py, pz, cx, cy, cz, nx, ny, nz, tCorner)
    local turn, la, lb = cornerTurn(px, py, pz, cx, cy, cz, nx, ny, nz)
    if not turn or turn < MIN_ARC_ANGLE then return 0, speed end
    local tanHalf = tan(turn * HALF)
    local radius = min(kneeRadius, ARC_LEG_SHARE * min(la, lb) / tanHalf)
    for _ = 0, MAX_ARC_HALVINGS do
        if radius <= MIN_LENGTH then break end
        local ex, ey, ez, _, _, _, ox, oy, oz, kx, ky, kz = cornerArc(px, py, pz, cx, cy, cz, nx, ny, nz, radius)
        if not ex then break end
        if arcIsClear(ex, ey, ez, ox, oy, oz, kx, ky, kz, turn, radius, tCorner - radius * tanHalf / speed) then
            return radius, min(speed, turnSpeed(shipAccel, shipDrag, radius, shipTopSpeed))
        end
        radius = radius * HALF
    end
    return 0, 0
end

return Corners
