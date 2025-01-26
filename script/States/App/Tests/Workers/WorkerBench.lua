local WorkerBench = require('States.Application')

function WorkerBench:onInit()
    Log.Info("WorkerBench:onInit: Start")

    local instancesCount = 4
    local messagesCount = 10000
    local taskIds = {}

    Profiler.Enable()
    Profiler.Begin('WorkerBench')

    ---@class WorkerId
    ---@field TestWorker integer Enum for the TestWorker worker
    WorkerId.Register({ "TestWorker" })

    TaskQueue:startWorker(WorkerId.TestWorker, "TestWorker", "script/States/App/Tests/Workers/TestWorkerFunction.lua", instancesCount)

    for i = 1, messagesCount do
        local taskId = TaskQueue:sendTask(WorkerId.TestWorker, "TestPayload")
        table.insert(taskIds, taskId)
        -- Log.Debug("New task: " .. tostring(taskId))
    end

    Log.Debug("Messages sent: " .. #taskIds)

    while #taskIds > 0 do
        local taskId, _ = TaskQueue:nextTaskResult(WorkerId.TestWorker)
        if taskId ~= nil then
            -- Log.Debug("Received: " .. tostring(taskId))
            table.removeValue(taskIds, taskId)
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
