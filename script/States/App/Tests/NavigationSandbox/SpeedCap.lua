-- Three ships, far apart. The first flies a long way under a speed cap of half of top speed; a
-- flight above it fails the run. The second is sent through points at half of top speed and
-- brakes for them; a pass above that speed fails the run. The third passes a right-angle corner
-- at 0.3 of top speed: the tightest turn it flies is logged, not judged.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "SpeedCap",
    timeLimit = 60,
    ships = {
        {
            start = { 0, 0, 0 },
            target = { 0, 0, -60000 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Seek",
                through = { { 0, 0, -30000 } },
                passRadiusM = 600,
                speedCapShare = 0.5,
            },
        },
        {
            start = { -40000, 0, 0 },
            target = { -40000, 0, -50000 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Seek",
                through = { { -40000, 0, -15000 }, { -40000, 0, -30000 } },
                passRadiusM = 600,
                passSpeedShare = 0.5,
            },
        },
        {
            start = { 40000, 0, 0 },
            target = { 60000, 0, -20000 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Seek",
                through = { { 40000, 0, -20000 } },
                passRadiusM = 600,
                passSpeedShare = 0.3,
                measureTurn = true,
            },
        },
    },
}
