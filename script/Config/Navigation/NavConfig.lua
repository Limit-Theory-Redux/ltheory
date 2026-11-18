-- Values the navigation orders fly by. Authored in metres, converted to game units on use.
-- Design: doc/script/nav/navigation.md

local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class NavConfig
local NavConfig = {
    -- m: a ship has arrived at a point (a position target, a patrol point) within this distance of it.
    pointArrivalM = 100,
    -- The distance from a target's centre at which a ship stops, in target radii, when nothing says otherwise.
    standOffRadii = 1.5,
    -- How much longer a planned path is than the straight line, for estimates, which plan nothing.
    -- Taken from the planned paths of sparse systems; a crowded system detours more.
    estimateDetourFactor = 1.02,
    -- m: a ship that has not come this much closer to its goal for `blockedWindowS` is stalled.
    blockedProgressM = 10,
    -- s
    blockedWindowS = 10,
    -- s: how often an order asks for a fresh plan while it flies one (an event may ask sooner). Each
    -- order's timer is offset by a share of it, so the requests of many orders spread over frames.
    replanPeriodS = 1,
    -- A goal that no plan rides on (a ship) has drifted when it has moved more than this share of the
    -- distance from the ship to it, from where the plan ends.
    goalDriftShare = 0.1,
}

---How close to a point a ship counts as arrived.
---@return gameunit
function NavConfig.pointArrival()
    return ScaleConfig:toGameUnits(NavConfig.pointArrivalM, "starSystem")
end

---How much closer to its goal a ship must come to count as making progress.
---@return gameunit
function NavConfig.blockedProgress()
    return ScaleConfig:toGameUnits(NavConfig.blockedProgressM, "starSystem")
end

return NavConfig
