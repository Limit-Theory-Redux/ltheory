-- Ships cross a system of bodies on circles: a star, two planets and a moon of the outer planet.
-- Each ship is planned a path on a worker that predicts where the bodies will be when the ship
-- gets there, while the ship already flies at its goal. The plan must come back with status Ok.
-- The planets are fast, so the bodies move far in the seconds a crossing takes:
--   ship 1 crosses the star, which is static: the plan detours around it;
--   ship 2 flies toward where the inner planet is now, but the planet is gone when it arrives:
--     no detour;
--   ship 3 flies where the moon is not yet, but will be when the ship arrives: a detour.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "SystemCrossing",
    timeLimit = 60,
    safeRadiusFactor = 1.5,
    obstacles = {
        -- 1: the star
        { radius = 3000, position = { 0, 0, 0 } },
        -- 2: the inner planet
        { radius = 1500, circle = { centre = { 0, 0, 0 }, radius = 25000, periodS = 90 } },
        -- 3: the outer planet
        { radius = 2000, circle = { centre = { 0, 0, 0 }, radius = 60000, periodS = 600 } },
        -- 4: the moon of the outer planet
        { radius = 500, circle = { around = 3, radius = 9000, periodS = 50 } },
    },
    ships = {
        {
            start = { 500, 0, -22000 },
            target = { 500, 0, 22000 },
            behaviour = { kind = "Plan", passRadiusM = 600 },
        },
        {
            start = { 50000, 0, 0 },
            target = { 10000, 0, 0 },
            behaviour = { kind = "Plan", passRadiusM = 600 },
        },
        {
            start = { 103000, 0, 6830 },
            target = { 35000, 0, 6830 },
            behaviour = { kind = "Plan", passRadiusM = 600 },
        },
    },
}
