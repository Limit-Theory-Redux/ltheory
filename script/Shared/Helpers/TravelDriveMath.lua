--- Travel-drive maths: the speed cap a zone puts on the drive, and the braking law that brings a
--- ship in on a goal. Pure numbers in and out (no entities, no config, no globals), so the game's
--- systems and the system planner kernel, which runs on a worker, share one implementation.
--- Design: doc/script/zones.md, doc/script/nav/system-planner.md.
local log, min, max = math.log, math.min, math.max

local TravelDriveMath = {}

-- A zone's drive cap starts to fall this many zone radii away from its body.
local INFLUENCE_RADIUS_FACTOR = 2.0
-- Below this drive multiplier the drive is released and normal flight does the rest.
local HANDOVER_MULT = 2
-- The braking time is this many times the sum of the drive multiplier's and the velocity's lag
-- times. The loop from the distance to the wanted speed is third order, and a margin of 4 gives it
-- real roots, so the ship decays onto the goal without overshooting; 2 oscillates.
local BRAKING_LAG_MARGIN = 4
-- Fixed-point steps that solve the decay rate of the braking law (see `decayRate`).
local DECAY_RATE_ITERATIONS = 12

TravelDriveMath.INFLUENCE_RADIUS_FACTOR = INFLUENCE_RADIUS_FACTOR
TravelDriveMath.HANDOVER_MULT = HANDOVER_MULT

--- The drive multiplier one zone allows at `distance` from its body: `openCap` from twice the zone
--- radius out, falling with the square of the distance to `zoneMin` at the body's centre.
---@param openCap number The multiplier in open space
---@param zoneMin number The multiplier at the body's centre
---@param zoneRadius gameunit
---@param distance gameunit From the body's centre
---@return number multiplier
function TravelDriveMath.zoneCap(openCap, zoneMin, zoneRadius, distance)
    local influence = zoneRadius * INFLUENCE_RADIUS_FACTOR
    if distance >= influence then return openCap end
    local t = distance / influence
    return min(openCap, zoneMin + (openCap - zoneMin) * t * t)
end

--- The time constant of the braking law: the wanted speed is the distance to the goal over it.
---@param rampDownSpeed number The drive multiplier's fall rate, per second
---@param linearDrag number The ship's drag rate, per second
---@return number seconds
function TravelDriveMath.brakingTime(rampDownSpeed, linearDrag)
    return BRAKING_LAG_MARGIN * (1 / rampDownSpeed + 1 / linearDrag)
end

--- The drive multiplier the braking law wants at a distance from the goal. The wanted speed is
--- `goalDist / brakingTime`, and full drive thrust settles at `multiplier · maxSpeed`.
---@param goalDist gameunit
---@param maxSpeed number The ship's top speed without the drive
---@param brakingTime number
---@return number
function TravelDriveMath.brakingMultiplier(goalDist, maxSpeed, brakingTime)
    return goalDist / (brakingTime * maxSpeed)
end

--- The drive multiplier the drive is held to: what the braking law wants, at least 1 (the drive
--- never slows a ship below its normal top speed) and at most what the zone allows.
---@param wanted number From `brakingMultiplier`
---@param zoneMax number The multiplier the zone allows
---@return number
function TravelDriveMath.cappedMultiplier(wanted, zoneMax)
    return min(zoneMax, max(1, wanted))
end

--- The rate (1/s) at which the distance to the goal decays under the braking law. The drive
--- multiplier and the velocity lag the falling cap, so a ship sits ahead of the wanted speed
--- and closes faster than `1 / brakingTime`. For a decay `e^(-r·t)` a first-order lag `τ`
--- scales its input by `1 / (1 - r·τ)`, so `r` solves `r = 1 / (T · (1 - r·τ₁) · (1 - r·τ₂))`;
--- the iteration contracts (factor under 0.5 at `BRAKING_LAG_MARGIN` 4).
---@param rampDownSpeed number
---@param linearDrag number
---@return number rate
function TravelDriveMath.decayRate(rampDownSpeed, linearDrag)
    local driveLag, velocityLag = 1 / rampDownSpeed, 1 / linearDrag
    local gain = 1 / TravelDriveMath.brakingTime(rampDownSpeed, linearDrag)
    local rate = gain
    for _ = 1, DECAY_RATE_ITERATIONS do
        rate = gain / ((1 - driveLag * rate) * (1 - velocityLag * rate))
    end
    return rate
end

--- Seconds until the ship is within `arrivalRadius` of the goal, from the braking law: a cruise at
--- the zone's cap while the law wants more, an exponential decay down to the hand-over distance or
--- the arrival radius (whichever is larger), then the flight at `maxSpeed`. Ramp-up and charging
--- time are not counted.
---@param goalDist gameunit Distance to the goal, 0 or more
---@param arrivalRadius gameunit
---@param maxSpeed number The ship's top speed without the drive
---@param rampDownSpeed number
---@param linearDrag number
---@param zoneMax number The drive multiplier the zone allows
---@param driveUsed boolean Whether the drive is, or will be, engaged
---@return number seconds
function TravelDriveMath.travelTime(goalDist, arrivalRadius, maxSpeed, rampDownSpeed, linearDrag, zoneMax, driveUsed)
    local lag = TravelDriveMath.brakingTime(rampDownSpeed, linearDrag)
    local handoverDist = HANDOVER_MULT * lag * maxSpeed
    local decayEnd = max(handoverDist, arrivalRadius)
    if not driveUsed or goalDist <= decayEnd then
        return max(0, goalDist - arrivalRadius) / maxSpeed
    end
    local cruiseSpeed = maxSpeed * max(1, zoneMax)
    -- Beyond `cruiseSpeed · lag` the law wants more than the cap, and the ship flies at the cap.
    local decayTop = min(goalDist, max(cruiseSpeed * lag, decayEnd))
    local seconds = max(0, handoverDist - arrivalRadius) / maxSpeed
    if goalDist > decayTop then
        seconds = seconds + (goalDist - decayTop) / cruiseSpeed
    end
    if decayTop > decayEnd then
        seconds = seconds + log(decayTop / decayEnd) / TravelDriveMath.decayRate(rampDownSpeed, linearDrag)
    end
    return seconds
end

return TravelDriveMath
