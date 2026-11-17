-- Values the navigation orders fly by. Authored in metres, converted to game units on use.
-- Design: doc/script/nav/navigation.md

local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class NavConfig
local NavConfig = {
    -- m: a ship has arrived at a point (a position target, a patrol point) within this distance of it.
    pointArrivalM = 100,
}

---How close to a point a ship counts as arrived.
---@return gameunit
function NavConfig.pointArrival()
    return ScaleConfig:toGameUnits(NavConfig.pointArrivalM, "starSystem")
end

return NavConfig
