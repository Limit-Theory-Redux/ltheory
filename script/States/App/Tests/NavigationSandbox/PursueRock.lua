-- A rock circles in the xz plane, moving toward +z where the scenario starts. Two ships pursue
-- it to a stand-off from its hull: one starts ahead of it and meets it head on, the other starts
-- behind it and chases it. Each is on station when it holds the stand-off for five seconds.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "PursueRock",
    timeLimit = 60,
    obstacles = {
        { radius = 300, circle = { centre = { 0, 0, -8000 }, radius = 4000, periodS = 120 } },
    },
    ships = {
        {
            start = { 4000, 0, 2000 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Pursue", obstacle = 1, standOffM = 200 },
        },
        {
            start = { 4000, 0, -16000 },
            facing = { 0, 0, 1 },
            behaviour = { kind = "Pursue", obstacle = 1, standOffM = 200 },
        },
    },
}
