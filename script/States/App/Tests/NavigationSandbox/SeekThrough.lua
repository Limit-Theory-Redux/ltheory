-- One ship is sent through three points with Seek, which does not brake for them, and then stops
-- at a target with Arrive. A pass below half of top speed fails the run.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "SeekThrough",
    timeLimit = 60,
    ships = {
        {
            start = { 0, 0, 0 },
            target = { 0, 0, -50000 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Seek",
                through = { { 3000, 0, -9000 }, { -3000, 0, -20000 }, { 0, 2000, -32000 } },
                passRadiusM = 600,
            },
        },
    },
}
