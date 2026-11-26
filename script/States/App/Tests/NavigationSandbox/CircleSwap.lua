-- Ships on a circle each fly to the opposite point, so all of them meet at the centre.
-- Hulls alternate between Solo and Medium, so ships of different sizes meet there.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local SHIP_COUNT = 8
local RADIUS_M   = 5000
local HULLS      = { Enums.ShipHulls.Solo, Enums.ShipHulls.Medium }

local ships = {}
for i = 0, SHIP_COUNT - 1 do
    local angle = 2 * math.pi * i / SHIP_COUNT
    local x, z = RADIUS_M * math.cos(angle), RADIUS_M * math.sin(angle)
    table.insert(ships, {
        start = { x, 0, z },
        target = { -x, 0, -z },
        hull = HULLS[i % #HULLS + 1],
    })
end

return NavigationSandbox.ScenarioApp {
    name = "CircleSwap",
    ships = ships,
}
