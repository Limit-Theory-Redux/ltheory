-- Orders that plan again on events, flown by `Nav.order`:
--   ship 1: a MoveTo to a point, which ship 2 chases;
--   ship 2: a MoveTo to ship 1. The plan does not ride a ship, so it is made again once ship 1 has moved
--     a tenth of the distance between them: the order is announced replanned, and the ship arrives;
--   ship 3: a MoveTo through a cost zone that is cheap to cross, which becomes dear a moment after the start.
--     The order is announced replanned, and the ship goes around the zone.
-- The ships of the first two are far from ship 3.

local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

-- The gap between the hulls that ship 2 stops at, in metres: a ship's own stand-off is hundreds of kilometres.
local STAND_OFF_GAP_M = 5000
-- The zone costs nothing worth a detour until the change.
local CHEAP_COST = 0.1
local DEAR_COST = 5
local COST_CHANGE_AFTER_S = 1

return NavigationSandbox.ScenarioApp {
    name = "OrdersReplan",
    timeLimit = 60,
    zones = {
        -- 1: crossed by ship 3's way
        { position = { 200000, 0, 0 }, radius = 12000, pathCost = CHEAP_COST,
            costChange = { afterS = COST_CHANGE_AFTER_S, pathCost = DEAR_COST } },
    },
    ships = {
        {
            start = { 0, 0, 0 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Orders", orders = { { kind = "MoveTo", point = { 0, 0, -30000 } } } },
        },
        {
            start = { 40000, 0, 0 },
            facing = { -1, 0, 0 },
            expect = { replanned = 1 },
            behaviour = { kind = "Orders", orders = { { kind = "MoveTo", ship = 1, gapM = STAND_OFF_GAP_M } } },
        },
        {
            start = { 200000, 0, -40000 },
            facing = { 0, 0, 1 },
            expect = { zone = "avoid", replanned = 1 },
            behaviour = { kind = "Orders", orders = { { kind = "MoveTo", point = { 200000, 0, 40000 } } } },
        },
    },
}
