-- The client of the estimate worker: sends a scene, a ship and goals, and hands back the estimates
-- when they have come. One worker serves the whole Lua state; it starts with the first request.
-- Design: doc/script/nav/navigation.md
local NavEstimate = require("Shared.Helpers.NavEstimate")

local WORKER_NAME   = "NavEstimate"
local WORKER_SCRIPT = "script/Shared/Helpers/NavEstimateWorker.lua"
local WORKER_COUNT  = 1

---What a request carries: the worker's task. See `NavEstimateWorker.lua`.
---@class NavEstimateRequest: NavEstimateScene
---@field ship NavEstimateShip
---@field start number[]
---@field goals number[]
---@field goalCount integer

---@class NavEstimateClient
local NavEstimateClient = {}

local workerId = nil
---Replies that have come and not been taken, by task id.
---@type table<integer, NavEstimateResult[]>
local results = {}
---Tasks nobody waits for any more, whose replies are dropped when they come.
---@type table<integer, true>
local discarded = {}

---Starts the worker once. The engine stops it at shutdown.
local function worker()
    if not workerId then
        workerId = TaskQueue:startWorker(WORKER_NAME, WORKER_SCRIPT, WORKER_COUNT)
        assert(workerId, "NavEstimateClient: the estimate worker did not start")
    end
    return workerId
end

---Sends a request. Not blocking.
---@param task NavEstimateRequest
---@return integer taskId To `take` the estimates with, or to `discard` them
function NavEstimateClient.request(task)
    local taskId = TaskQueue:sendTask(worker(), task)
    assert(taskId, "NavEstimateClient: the estimate request was not sent")
    return taskId
end

---@param flat number[] `NavEstimate.RESULT_STRIDE` numbers per goal
---@return NavEstimateResult[]
local function unpack(flat)
    local stride, field, out = NavEstimate.RESULT_STRIDE, NavEstimate.Result, {}
    for i = 1, #flat, stride do
        if flat[i + field.Reachable] == 0 then
            out[#out + 1] = { reachable = false }
        else
            out[#out + 1] = { reachable = true, time = flat[i + field.Time], distance = flat[i + field.Distance],
                risk = flat[i + field.Risk] }
        end
    end
    return out
end

---Collects the replies that have come since the last call. Not blocking. Call it before `take`.
---A worker's error is raised here: an estimate that fails is a bug, not an answer.
function NavEstimateClient.poll()
    if not workerId then return end
    while true do
        local taskId, value, err = TaskQueue:nextTaskResult(workerId)
        if not taskId then return end
        if discarded[taskId] then
            discarded[taskId] = nil
        else
            assert(not err, "NavEstimateClient: the estimate worker failed: " .. tostring(err))
            results[taskId] = unpack(value.values)
        end
    end
end

---The estimates of a request, once they have come and been collected by `poll`.
---@param taskId integer
---@return NavEstimateResult[]|nil estimates Nil while the reply has not come
function NavEstimateClient.take(taskId)
    local found = results[taskId]
    results[taskId] = nil
    return found
end

---Drops a request nobody waits for: its reply is thrown away, whether it has come or not.
---@param taskId integer
function NavEstimateClient.discard(taskId)
    if results[taskId] then
        results[taskId] = nil
    else
        discarded[taskId] = true
    end
end

return NavEstimateClient
