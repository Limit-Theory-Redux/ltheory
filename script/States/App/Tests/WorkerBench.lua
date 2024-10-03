local WorkerBench = require('States.Application')

local function removeElement(t, value)
    for i, v in ipairs(t) do
        if v == value then
            table.remove(t, i)
            return
        end
    end
    for i, v in ipairs(t) do
        print(tostring(i) .. ": " .. tostring(v))
    end
    Log.Error("Cannot find table value " .. tostring(value))
end

function WorkerBench:onInit()
    Log.Info("WorkerBench:onInit: Start")

    local instancesCount = 4
    local messagesCount = 1000
    local taskIds = {}

    Profiler.Enable()
    Profiler.Begin('WorkerBench')

    WorkerId.AddWorkers({ "TestWorker" })
    if TaskQueue:startWorker(WorkerId.TestWorker, "TestWorker", "script/States/App/Tests/TestWorkerFunction.lua", instancesCount) == false then
        Log.Error("Cannot start worker")
    end

    for i = 1, messagesCount do
        local taskId = TaskQueue:sendTask(WorkerId.TestWorker, "TestPayload")
        if taskId == nil then
            Log.Error("Cannot send task " .. i)
        end
        table.insert(taskIds, taskId)
        -- Log.Debug("New task: " .. tostring(taskId))
    end

    Log.Debug("Messages sent: " .. #taskIds)

    while #taskIds > 0 do
        local taskId, _ = TaskQueue:nextTaskResult(WorkerId.TestWorker)
        if taskId ~= nil then
            -- Log.Debug("Received: " .. tostring(taskId))
            removeElement(taskIds, taskId)
            -- Log.Debug("Left: " .. tostring(#taskIds))
        end
    end

    TaskQueue:stopAllWorkers()

    Profiler.End()
    Profiler.Disable()

    Log.Info("WorkerBench:onInit: End")
end

function WorkerBench:onPreRender() end

function WorkerBench:onRender() end

function WorkerBench:onPostRender() end

return WorkerBench
