-- One ship flies to a point behind it and to one side: it turns, flies and stops.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "SingleShip",
    ships = {
        { start = { 0, 0, 0 }, target = { 6000, 0, 8000 }, facing = { 0, 0, -1 } },
    },
}
