local WorkerBench = require('States.Application')

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
        table.insert(taskIds, tonumber(taskId), taskId)
    end

    Log.Debug("Messages sent: " .. #taskIds)

    while #taskIds > 0 do
        local taskId, _ = TaskQueue:nextTaskResult(WorkerId.TestWorker)
        if taskId ~= nil then
            table.remove(taskIds, tonumber(taskId))
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
