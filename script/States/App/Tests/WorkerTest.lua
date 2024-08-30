local WorkerTest = require('States.Application')

function WorkerTest:onInit()
    Worker.AddWorkers({ "TestWorker" })
    if TaskQueue:startWorker(Worker.TestWorker, "TestWorker", "script/States/App/Tests/TestWorkerFunction.lua") == false then
        Log.Error("Cannot start worker")
    end
    local expectedTaskId = TaskQueue:sendTask(Worker.TestWorker, "TestPayload")
    if expectedTaskId == nil then
        Log.Error("Cannot send task")
    end

    local taskId, payload = TaskQueue:nextTaskResult(Worker.TestWorker)
    -- while taskId == nil do
    --     taskId, payload = TaskQueue:nextTaskResult(Worker.TestWorker)
    -- end

    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(payload == "TestPayload", "Expected 'TestPayload' but was '" .. payload .. "'")
end

function WorkerTest:onPreRender() end

function WorkerTest:onRender() end

function WorkerTest:onPostRender() end

return WorkerTest
