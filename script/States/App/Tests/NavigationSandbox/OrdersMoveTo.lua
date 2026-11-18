-- Ships flown by `Nav.order`, with the orders that end: each is done when its order is.
--   ship 1: MoveTo a body on a circle: the plan ends at the entity's approach, which is a stand-off from
--     the body, wherever the body is by then;
--   ship 2: Patrol through three points without looping, past a body that sits on its second leg;
--   ship 3: MoveTo a point and then Hold, which keeps the ship where it is and never ends: the ship is
--     judged by holding its place at rest.
-- Ship 1 starts at the orders' own stand-off from a body, hundreds of kilometres; the points are tens.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "OrdersMoveTo",
    timeLimit = 90,
    obstacles = {
        -- 1: circles the origin
        { radius = 100000, circle = { centre = { 0, 0, 0 }, radius = 300000, periodS = 1500 } },
        -- 2: on the second leg of ship 2's patrol
        { radius = 5000, position = { 2060000, 0, 0 } },
    },
    ships = {
        {
            start = { 0, 0, -800000 },
            facing = { 0, 0, 1 },
            behaviour = { kind = "Orders", orders = { { kind = "MoveTo", obstacle = 1 } } },
        },
        {
            start = { 2000000, 0, 0 },
            facing = { 1, 0, 0 },
            behaviour = {
                kind = "Orders",
                orders = { { kind = "Patrol", points = { { 2040000, 0, 0 }, { 2080000, 0, 0 }, { 2080000, 0, 40000 } } } },
            },
        },
        {
            start = { -2000000, 0, 0 },
            facing = { 1, 0, 0 },
            behaviour = {
                kind = "Orders",
                orders = { { kind = "MoveTo", point = { -1950000, 0, 0 } }, { kind = "Hold" } },
            },
        },
    },
}
