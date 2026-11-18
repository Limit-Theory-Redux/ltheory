-- A ship flown by a Routine MoveTo is displaced mid-flight by a Combat MoveTo given later: the first is
-- suspended, the Combat one is flown and ends, and the first resumes from where the ship is and ends.
-- The ship is judged by the first order: it has arrived when that order has been suspended, then done.
-- The distances are those of the orders' own stand-off, hundreds of kilometres.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local INTERRUPT_AFTER_S = 3

return NavigationSandbox.ScenarioApp {
    name = "OrdersPriority",
    timeLimit = 60,
    obstacles = {},
    ships = {
        {
            start = { 0, 0, 0 },
            facing = { 0, 0, 1 },
            behaviour = {
                kind = "Orders",
                orders = {
                    { kind = "MoveTo", point = { 0, 0, 200000 }, judged = true, suspends = true },
                    {
                        kind = "MoveTo", point = { 60000, 0, 60000 }, priority = "Combat",
                        afterS = INTERRUPT_AFTER_S,
                    },
                },
            },
        },
    },
}
