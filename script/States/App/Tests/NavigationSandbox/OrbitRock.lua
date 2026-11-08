-- A rock circles in the xz plane. Two ships fly to circles about it, in the plane normal to y,
-- and go once round them: one starts in the plane, the other above it. The circles have different
-- radii, so the ships do not share one.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "OrbitRock",
    timeLimit = 60,
    obstacles = {
        { radius = 500, circle = { centre = { 0, 0, -10000 }, radius = 3000, periodS = 200 } },
    },
    ships = {
        {
            start = { 3000, 0, -2000 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Orbit", obstacle = 1, radiusM = 1500, normal = { 0, 1, 0 } },
        },
        {
            start = { -3000, 4000, -10000 },
            facing = { 1, 0, 0 },
            behaviour = { kind = "Orbit", obstacle = 1, radiusM = 2500, normal = { 0, 1, 0 } },
        },
    },
}
