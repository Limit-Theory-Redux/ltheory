-- One ship flies toward a far point and is switched to Hold at half of top speed: it brakes,
-- overshoots the point it was at, comes back and stops there.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "HoldAfterCruise",
    timeLimit = 60,
    ships = {
        {
            start = { 0, 0, 0 },
            target = { 0, 0, -60000 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Hold", holdAtSpeed = 0.5 },
        },
    },
}
