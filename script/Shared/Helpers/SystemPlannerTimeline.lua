-- System planner kernel, part 4 of 6: the travel drive of a plan, and the timeline of a leg flown
-- under it, where that is sooner than the planning speed. Not reentrant: it keeps module scratch
-- state, set by `setDrive`.
-- Design: doc/script/nav/system-planner.md.

local TravelDriveMath = require("Shared.Helpers.TravelDriveMath")
local Layout          = require("Shared.Helpers.SystemPlannerLayout")
local Profile         = require("Shared.Helpers.SystemPlannerProfile")
local Scene           = require("Shared.Helpers.SystemPlannerScene")

local exp = math.exp
local max, min, abs = math.max, math.min, math.abs

local DRIVE_OPEN_CAP, DRIVE_CHARGE, DRIVE_RAMP_UP = Layout.DRIVE_OPEN_CAP, Layout.DRIVE_CHARGE, Layout.DRIVE_RAMP_UP
local DRIVE_RAMP_DOWN, DRIVE_MIN_LENGTH = Layout.DRIVE_RAMP_DOWN, Layout.DRIVE_MIN_LENGTH
local HALF, isFinite = Layout.HALF, Layout.isFinite
local zoneState, DRIVE_FEATURE_STEP = Scene.zoneState, Scene.DRIVE_FEATURE_STEP
local flatProfile = Profile.flat

local Timeline = {}

-- While the drive's multiplier or the ship's speed is off its target by more than this share, a
-- step is at most `DRIVE_TRANSIENT_STEP` of the shortest time constant that governs them.
local DRIVE_TRANSIENT_TOLERANCE = 1e-3
local DRIVE_TRANSIENT_STEP = 0.1
-- The shortest step (seconds), which ends a search that would creep, and the most steps of one
-- leg, past which the leg is flown without the drive.
local DRIVE_MIN_STEP_S = 1e-6
local MAX_DRIVE_STEPS = 20000

-- The planning speed and the ship's linear drag rate of the running plan.
local speed, shipDrag
-- The travel drive: whether it is planned, the multiplier in open space, the charge time, the
-- rise and fall rates, the shortest time constant of the drive and the ship's speed, the braking
-- law's time constant and the distance from a leg's end at which the drive is released, and the
-- length below which a leg is not driven.
local driveOn, driveOpenCap, driveCharge, driveRampUp, driveRampDown = false, 1, 0, 1, 1
local driveMinLength = 0
local driveShortestLag, driveBrakingTime, driveRelease = 1, 1, 0

---Sets the drive scratch from the drive block, or turns the drive off for none. The drive needs a
---ship with drag (its braking law divides by it). `planSpeed` is the ship's speed without the drive.
---@return boolean valid
function Timeline.setDrive(drive, planSpeed, drag)
    speed, shipDrag = planSpeed, drag
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

---Whether the plan has a drive.
function Timeline.isOn() return driveOn end

---The drive multiplier in open space.
function Timeline.openCap() return driveOpenCap end

---The length below which a leg that does not continue a run is not driven.
function Timeline.minLength() return driveMinLength end

---The distance from a leg's end at which the drive is released.
function Timeline.release() return driveRelease end

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
        local limit, dt = zoneState(driveOpenCap, ax + ux * s, ay + uy * s, az + uz * s, t, v)
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
---@return boolean notTimed True when a leg long enough to be driven could not be timed under the drive
function Timeline.buildProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run)
    local carried = run and run.m
    local notTimed = false
    if driveOn and (carried or length >= driveMinLength) then
        local timed = driveProfile(profile, t0, length, ax, ay, az, ux, uy, uz, run and run.beyond or 0, carried,
            run and run.v)
        if timed and profile.t[profile.count] < t0 + length / speed then return false end
        notTimed = not timed and length > driveRelease + speed * driveCharge
    end
    flatProfile(profile, t0, length, speed)
    return notTimed
end

return Timeline
