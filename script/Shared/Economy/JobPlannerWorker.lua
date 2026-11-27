-- Worker script of the NPC captains' economic thinking: runs `JobPlanner.planAll` on a `TaskQueue`
-- worker thread, off the main thread. A task is a snapshot (see `JobPlanner`); the result is a flat
-- array of numbers, `JobPlanner.Layout.Result.Stride` per agent.
-- Design: doc/script/economy.md
package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")
local JobPlanner     = require("Shared.Economy.JobPlanner")

---@param snapshot table
---@return table `values`: the answers
Run = WorkerFunction.Create(function(snapshot)
    return { values = JobPlanner.planAll(snapshot) }
end)
