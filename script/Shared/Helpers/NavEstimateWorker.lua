-- Worker script of the navigation estimates: runs `NavEstimate.many` on a `TaskQueue` worker thread.
-- A task is a table (see `Run`); the result is a flat array of numbers, `NavEstimate.RESULT_STRIDE`
-- per goal, so it crosses the payload without nested tables.
-- Design: doc/script/nav/navigation.md
package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")
local NavEstimate    = require("Shared.Helpers.NavEstimate")

local Result = NavEstimate.Result

---Estimates the flights from one start to many goals.
---@param request table `bodies`, `bodyCount`, `zones`, `zoneCount` (the scene), `ship` (a `NavEstimateShip`), `start` (3 numbers), `goals` and `goalCount`
---@return table `values`: `RESULT_STRIDE` numbers per goal, in the goals' order, with 0 for what an unreachable goal lacks
Run = WorkerFunction.Create(function(request)
    local results = NavEstimate.many(request, request.ship, request.start, request.goals, request.goalCount)
    local values = {}
    for i, result in ipairs(results) do
        local base = (i - 1) * NavEstimate.RESULT_STRIDE
        values[base + Result.Reachable + 1] = result.reachable and 1 or 0
        values[base + Result.Time + 1] = result.time or 0
        values[base + Result.Distance + 1] = result.distance or 0
        values[base + Result.Risk + 1] = result.risk or 0
    end
    return { values = values }
end)
