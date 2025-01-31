local WorkerTest = require('States.Application')

function WorkerTest:onInit()
    Log.Info("WorkerTest:onInit: Start")

    local workerId1 = TaskQueue:startWorker("TestWorker", "script/States/App/Tests/TestWorkerFunction.lua", 1)
    local workerId2 = TaskQueue:startWorker("TestWorker2", "script/States/App/Tests/TestWorkerFunction2.lua", 1)

    -- Simple test
    local expectedTaskId = TaskQueue:sendTask(workerId1, "TestPayload")
    local taskId, payload = TaskQueue:nextTaskResult(workerId1)
    while taskId == nil do
        taskId, payload = TaskQueue:nextTaskResult(workerId1)
    end

    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(payload == "TestPayload_OUT", "Expected 'TestPayload_OUT' but was '" .. payload .. "'")

    -- Complex test
    local expectedPayload = {
        boolVal = true,
        intVal = 3,
        floatVal = 4.0,
        strVal = "TestPayload2",
        tableVal = {
            boolVal = true,
            intVal = 5,
            floatVal = 6.0,
            strVal = "TestPayload3",
        }
    }
    local expectedTaskId = TaskQueue:sendTask(workerId2, expectedPayload)
    local taskId, payload = TaskQueue:nextTaskResult(workerId2)
    while taskId == nil do
        taskId, payload = TaskQueue:nextTaskResult(workerId2)
    end

    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(table.equal(payload, expectedPayload, true),
        "Expected '" .. table.tostring(expectedPayload, true) .. "' but was '" .. table.tostring(payload, true) .. "'")

    TaskQueue:stopAllWorkers()

    Log.Info("WorkerTest:onInit: End")
end

function WorkerTest:onPreRender() end

function WorkerTest:onRender() end

function WorkerTest:onPostRender() end

return WorkerTest
