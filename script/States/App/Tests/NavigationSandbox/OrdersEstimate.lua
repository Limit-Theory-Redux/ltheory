-- Destinations are scored with estimates, a ship is sent to the best, interrupted by a higher-priority order,
-- and the first order resumes and ends.
--   ship 1 is under a cautious cost profile, which weighs a belt (a cost zone) ten times. Its candidates:
--     1: the nearest, inside a station's hull: not reachable, skipped;
--     2: behind the belt: sooner than 3, but the belt's weighted risk loses it the score;
--     3: farther, clear of the belt: wins.
--   Its MoveTo to the winner is interrupted by a Combat MoveTo, so it is suspended; it resumes and is done.
--   ship 2, far from the rest, chooses among one candidate and flies it undisturbed: its estimated time is
--   logged against the time it took (not judged).
-- The distances are hundreds of kilometres, as those of the other orders' scenarios.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local INTERRUPT_AFTER_S = 3
-- Seconds one metre of weighted risk is worth in the score.
local RISK_WEIGHT_S_PER_M = 1e-4

return NavigationSandbox.ScenarioApp {
    name = "OrdersEstimate",
    timeLimit = 90,
    obstacles = {
        -- 1: the station
        { position = { -30000, 0, 40000 }, radius = 3000, station = true },
    },
    zones = {
        -- 1: the belt
        { position = { 0, 0, 60000 }, radius = 20000, pathCost = 1, tags = { "AsteroidField" } },
    },
    ships = {
        {
            start = { 0, 0, 0 },
            facing = { 0, 0, 1 },
            costProfile = "Cautious",
            expect = { chosen = 3 },
            behaviour = {
                kind = "Orders",
                orders = {
                    {
                        kind = "MoveTo", judged = true, suspends = true,
                        choose = {
                            points = { { -30000, 0, 40000 }, { 0, 0, 100000 }, { 60000, 0, 130000 } },
                            riskWeightSPerM = RISK_WEIGHT_S_PER_M,
                        },
                    },
                    { kind = "MoveTo", point = { 60000, 0, 10000 }, priority = "Combat", afterS = INTERRUPT_AFTER_S },
                },
            },
        },
        {
            start = { 400000, 0, 0 },
            facing = { 0, 0, 1 },
            expect = { chosen = 1 },
            behaviour = {
                kind = "Orders",
                orders = {
                    { kind = "MoveTo", choose = { points = { { 400000, 0, 150000 } }, riskWeightSPerM = 0 } },
                },
            },
        },
    },
}
