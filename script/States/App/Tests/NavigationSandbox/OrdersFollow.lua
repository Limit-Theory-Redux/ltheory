-- Ships flown by `Nav.order`: a leader on a MoveTo, and two followers on a Follow order each, which never
-- ends. The followers start near their slots, which are given in the leader's axes (right, up, forward),
-- within the range from which a Follow order keeps its slot without a plan. Each is on station when it
-- holds its slot, at rest relative to the leader, for five seconds once the leader has finished its own
-- flight. None may touch the leader or each other.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "OrdersFollow",
    timeLimit = 60,
    ships = {
        {
            start = { 0, 0, 0 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Orders", orders = { { kind = "MoveTo", point = { 0, 0, -30000 } } } },
        },
        {
            start = { 1500, 0, 0 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Orders",
                orders = { { kind = "Follow", ship = 1, offsetM = { 1500, 0, 0 } } },
            },
        },
        {
            start = { 0, 0, 1500 },
            facing = { 0, 0, -1 },
            behaviour = {
                kind = "Orders",
                orders = { { kind = "Follow", ship = 1, offsetM = { 0, 0, -1500 } } },
            },
        },
    },
}
