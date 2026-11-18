-- Three ships fly the same way, 80 km through a belt (a cost zone tagged AsteroidField), each
-- under another cost profile. The planner goes around the belt when that is cheaper than crossing
-- it, which depends on how the ship's profile weighs the belt's tag:
--   ship 1, Cautious: the belt costs ten times its path cost, so the plan goes round it;
--   ship 2, Bold: the belt is free, so the plan goes straight through it;
--   ship 3 starts Bold and changes to Cautious once its first plan has come: the change asks for
--     a new plan from where the ship is, which goes round the belt.
-- Every plan must come back with status Ok.

local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "CostZoneReplan",
    timeLimit = 90,
    zones = {
        -- 1: the belt
        { position = { 0, 0, 0 }, radius = 12000, pathCost = 1, tags = { "AsteroidField" } },
    },
    ships = {
        {
            start = { -6000, 0, -40000 },
            target = { -6000, 0, 40000 },
            costProfile = "Cautious",
            behaviour = { kind = "Plan", passRadiusM = 600 },
            expect = { zone = "avoid", replans = 0 },
        },
        {
            start = { 0, 0, -40000 },
            target = { 0, 0, 40000 },
            costProfile = "Bold",
            behaviour = { kind = "Plan", passRadiusM = 600 },
            expect = { zone = "cross", replans = 0 },
        },
        {
            start = { 6000, 0, -40000 },
            target = { 6000, 0, 40000 },
            costProfile = "Bold",
            costProfileChange = { afterPlans = 1, to = "Cautious" },
            behaviour = { kind = "Plan", passRadiusM = 600 },
            expect = { zone = "avoid", replans = 1 },
        },
    },
}
