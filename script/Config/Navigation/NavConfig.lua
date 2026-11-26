-- Values the navigation orders fly by. Authored in metres, converted to game units on use.
-- Design: doc/script/nav/navigation.md

local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class NavConfig
local NavConfig = {
    -- m: a ship has arrived at a point (a position target, a patrol point) within this distance of it.
    pointArrivalM = 100,
    -- The distance from a planet's, a moon's or an asteroid's centre at which a ship stops, in target radii.
    standOffRadii = 1.5,
    -- m: the distance from the hull at which a ship stops at a target of no kind (a ship), when nothing
    -- says otherwise.
    genericGapM = 1000,
    -- m: a stand-off given in target radii is at least this far from the target's centre.
    minStandOffM = 500000,
    -- How much longer a planned path is than the straight line, for estimates, which plan nothing.
    -- Taken from the planned paths of sparse systems; a crowded system detours more.
    estimateDetourFactor = 1.02,
    -- m: a ship that has not come this much closer to its goal for `blockedWindowS` is stalled.
    blockedProgressM = 10,
    -- s
    blockedWindowS = 10,
    -- By rate class, in seconds: `avoidancePeriodS`, how often a ship's avoidance runs (0: at every
    -- step), and `replanPeriodS`, how often an order asks for a fresh plan while it flies one (an event
    -- may ask sooner). Each order's timer is offset by a share of its period, so the requests of many
    -- orders spread over frames.
    rateClasses = {
        [Enums.NavRateClass.High]   = { avoidancePeriodS = 0,    replanPeriodS = 0.5 },
        [Enums.NavRateClass.Normal] = { avoidancePeriodS = 0.02, replanPeriodS = 1 },
        [Enums.NavRateClass.Low]    = { avoidancePeriodS = 0.05, replanPeriodS = 2 },
    },
    -- s: how often a ship's rate class is judged again (a zone entry or exit judges it at once). Each
    -- ship's judging is offset by a share of it.
    rateJudgePeriodS = 1,
    -- m, and a count: a ship with at least `trafficShips` other ships within `trafficRadiusM` is in dense
    -- traffic, which gives it the High class.
    trafficRadiusM = 2000,
    trafficShips = 3,
    -- A goal that no plan rides on (a ship) has drifted when it has moved more than this share of the
    -- distance from the ship to it, from where the plan ends.
    goalDriftShare = 0.1,
    -- How the navigation world's ships avoid obstacles: `horizonS`, in seconds, ignores obstacles reached
    -- later than that; `clearanceRadii`, in ship radii, is the largest distance kept from an obstacle's
    -- surface; `aimFactor`, 1 or more, is the multiple of that clearance a new course passes at.
    avoidance = { horizonS = 10, clearanceRadii = 2, aimFactor = 1.05 },
    -- 0 to 1: a ship has arrived when it is within this share of the gap between the target's hull and
    -- its stand-off distance from the stand-off point.
    arrivalShare = 0.5,
    -- s: the travel drive is used only on a leg that a ship covers in at least this long at its top speed
    -- without the drive. It is also a follower's join range.
    driveMinLegTimeS = 60,
    -- More than 1: a tracking follower goes back to planning when its slot is farther than this many
    -- join ranges.
    followLeaveFactor = 2,
}

---How close to a point a ship counts as arrived.
---@return gameunit
function NavConfig.pointArrival()
    return ScaleConfig:toGameUnits(NavConfig.pointArrivalM, "starSystem")
end

---The least distance from a target's centre at which a stand-off given in radii stops a ship.
---@return gameunit
function NavConfig.minStandOff()
    return ScaleConfig:toGameUnits(NavConfig.minStandOffM, "starSystem")
end

---How much closer to its goal a ship must come to count as making progress.
---@return gameunit
function NavConfig.blockedProgress()
    return ScaleConfig:toGameUnits(NavConfig.blockedProgressM, "starSystem")
end

---How far from a ship the ships that make traffic dense are counted.
---@return gameunit
function NavConfig.trafficRadius()
    return ScaleConfig:toGameUnits(NavConfig.trafficRadiusM, "starSystem")
end

return NavConfig
