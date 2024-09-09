local WorkerTest = require('States.Application')

function WorkerTest:onInit()
    Log.Info("WorkerTest:onInit: Start")

    WorkerId.AddWorkers({ "TestWorker" })
    if TaskQueue:startWorker(WorkerId.TestWorker, "TestWorker", "script/States/App/Tests/TestWorkerFunction.lua", 1) == false then
        Log.Error("Cannot start worker")
    end
    local expectedTaskId = TaskQueue:sendTask(WorkerId.TestWorker, "TestPayload")
    if expectedTaskId == nil then
        Log.Error("Cannot send task")
    end

    local taskId, payload = TaskQueue:nextTaskResult(WorkerId.TestWorker)
    while taskId == nil do
        taskId, payload = TaskQueue:nextTaskResult(WorkerId.TestWorker)
    end

    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(payload == "TestPayload_OUT", "Expected 'TestPayload_OUT' but was '" .. payload .. "'")

    TaskQueue:stopAllWorkers()

    Log.Info("WorkerTest:onInit: End")
end

function WorkerTest:onPreRender() end

function WorkerTest:onRender() end

function WorkerTest:onPostRender() end

return WorkerTest
