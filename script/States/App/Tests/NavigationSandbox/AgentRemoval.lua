-- Agents come and go. Ship 1 flies along z with ship 2 behind it in the same lane, and ship 3 flies
-- a lane beside them. Ship 1 is removed from the navigation world, and its body coasts on and
-- slows down in the lane, so ship 2 must sense it as an obstacle from then on. Ship 1 has the
-- first slot: the others keep theirs. Ship 4 waits as a body at rest, off every lane, and joins
-- after the removal: it is added into the slot ship 1 freed and must fly to its target like the
-- others. No ship may touch another.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

-- Seconds of simulated time: the removal, and the join after it.
local REMOVE_S = 1.5
local JOIN_S   = 2.0

return NavigationSandbox.ScenarioApp {
    name = "AgentRemoval",
    ships = {
        { start = { 0, 0, 5000 }, target = { 0, 0, -10000 }, removeAtS = REMOVE_S },
        { start = { 0, 0, 10000 }, target = { 0, 0, -15000 } },
        { start = { 3000, 0, 10000 }, target = { 3000, 0, -15000 } },
        { start = { -6000, 0, -6000 }, target = { 6000, 0, 6000 }, joinAtS = JOIN_S, joinsSlotOf = 1 },
    },
}
