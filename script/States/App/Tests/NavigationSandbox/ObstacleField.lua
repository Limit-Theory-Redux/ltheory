-- Three ships fly abreast along x through static spheres: some sit on a ship's straight course,
-- the others beside the courses.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local HALF_RANGE_M = 10000
local SPACING_M    = 600

local function lane(z)
    return { start = { -HALF_RANGE_M, 0, z }, target = { HALF_RANGE_M, 0, z } }
end

return NavigationSandbox.ScenarioApp {
    name = "ObstacleField",
    ships = { lane(-SPACING_M), lane(0), lane(SPACING_M) },
    -- { x, y, z } and radius, in metres.
    obstacles = {
        -- On the courses.
        { position = { -4000, 0, 0 },     radius = 400 },
        { position = { -1000, 0, 300 },   radius = 500 },
        { position = { 2000, 0, -300 },   radius = 300 },
        { position = { 5000, 0, 0 },      radius = 800 },
        -- Beside the courses.
        { position = { 0, 0, 1500 },      radius = 800 },
        { position = { 3000, 0, -1500 },  radius = 600 },
    },
}
