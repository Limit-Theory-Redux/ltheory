-- Two streams of ships cross at right angles: one flies along x, the other along z, each ship
-- to the mirror of its start. The streams meet at the origin at the same time.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

-- Ships per stream, the gap between neighbours abreast, and the distance of every start from the origin, in metres.
local SHIPS_PER_STREAM = 4
local SPACING_M        = 800
local HALF_RANGE_M     = 10000

local ships = {}
for i = 1, SHIPS_PER_STREAM do
    -- Offset of this ship from the stream's axis, centred on it.
    local offset = (i - (SHIPS_PER_STREAM + 1) / 2) * SPACING_M
    table.insert(ships, { start = { -HALF_RANGE_M, 0, offset }, target = { HALF_RANGE_M, 0, offset } })
    table.insert(ships, { start = { offset, 0, -HALF_RANGE_M }, target = { offset, 0, HALF_RANGE_M } })
end

return NavigationSandbox.ScenarioApp {
    name = "CrossingStreams",
    ships = ships,
}
