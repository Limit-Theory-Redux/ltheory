-- Two ships face each other and fly to each other's start along the same line.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "HeadOn",
    ships = {
        { start = { -10000, 0, 0 }, target = { 10000, 0, 0 } },
        { start = { 10000, 0, 0 }, target = { -10000, 0, 0 } },
    },
}
