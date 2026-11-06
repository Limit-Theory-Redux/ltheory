-- A leader flies 15 km ahead and stops. Two followers start in their slots, one on each side of
-- it, and hold them: each is on station when it holds its slot for five seconds once the leader
-- has stopped. The slots are given in the leader's axes: right, up, forward.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

return NavigationSandbox.ScenarioApp {
    name = "FollowLeader",
    timeLimit = 60,
    ships = {
        { start = { 0, 0, 0 }, target = { 0, 0, -15000 }, facing = { 0, 0, -1 } },
        {
            start = { 1500, 0, 0 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Follow", ship = 1, offsetM = { 1500, 0, 0 } },
        },
        {
            start = { -1500, 0, 0 },
            facing = { 0, 0, -1 },
            behaviour = { kind = "Follow", ship = 1, offsetM = { -1500, 0, 0 } },
        },
    },
}
