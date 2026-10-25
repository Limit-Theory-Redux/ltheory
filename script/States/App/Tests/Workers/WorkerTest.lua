local WorkerTest = require('States.Application')

local function waitResult(workerId, what)
    local taskId, payload, err = TaskQueue:waitTaskResult(workerId, 10000)
    assert(taskId ~= nil, "Timeout waiting for " .. what)
    return taskId, payload, err
end

function WorkerTest:onInit()
    Log.Info("WorkerTest:onInit: Start")

    local taskId, payload, err, expectedTaskId

    -- Failure to start must not return an id
    local badId = TaskQueue:startWorker("TestWorkerMissing", "script/States/App/Tests/Workers/DoesNotExist.lua", 1)
    assert(badId == nil, "startWorker of a missing script must return nil")
    assert(TaskQueue:sendTask(badId, "x") == nil, "sendTask to an invalid worker must return nil, not crash")

    local workerId1 = TaskQueue:startWorker("TestWorker", "script/States/App/Tests/Workers/TestWorkerFunction.lua", 1)
    local workerId2 = TaskQueue:startWorker("TestWorker2", "script/States/App/Tests/Workers/TestWorkerFunction2.lua", 1)
    local workerId3 = TaskQueue:startWorker("TestWorker3", "script/States/App/Tests/Workers/TestWorkerFunction3.lua", 1)
    assert(workerId1 and workerId2 and workerId3, "Cannot start test workers")

    -- Polling with nothing ready must be instant
    local t0 = os.clock()
    for _ = 1, 20 do
        assert(TaskQueue:nextTaskResult(workerId1) == nil)
    end
    local pollTime = os.clock() - t0
    Log.Info("WorkerTest: 20 empty polls took %.2f ms", pollTime * 1000)
    assert(pollTime < 0.2, "nextTaskResult blocks: " .. tostring(pollTime))

    -- Simple test
    expectedTaskId = TaskQueue:sendTask(workerId1, "TestPayload")
    taskId, payload = waitResult(workerId1, "simple task")
    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(payload == "TestPayload_OUT", "Expected 'TestPayload_OUT' but was '" .. tostring(payload) .. "'")

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
    expectedTaskId = TaskQueue:sendTask(workerId2, expectedPayload)
    taskId, payload = waitResult(workerId2, "complex task")
    assert(expectedTaskId == taskId, "Expected " .. tostring(expectedTaskId) .. " but was " .. tostring(taskId))
    assert(table.equal(payload, expectedPayload, true),
        "Expected '" .. table.tostring(expectedPayload, true) .. "' but was '" .. table.tostring(payload, true) .. "'")

    -- Array round trips (bulk path), including a large one
    local big = {}
    for i = 1, 100000 do big[i] = i * 0.5 end
    local arrays = {
        numbers = { 1, 2.5, -3, 1e300 },
        bools = { true, false, true },
        strings = { "a", "bb", "" },
        big = big,
        nested = { inner = { 7, 8, 9 } },
    }
    expectedTaskId = TaskQueue:sendTask(workerId2, arrays)
    taskId, payload = waitResult(workerId2, "array task")
    assert(expectedTaskId == taskId)
    assert(#payload.numbers == 4 and payload.numbers[2] == 2.5 and payload.numbers[4] == 1e300, "numbers")
    assert(#payload.bools == 3 and payload.bools[1] == true and payload.bools[2] == false, "bools")
    assert(#payload.strings == 3 and payload.strings[2] == "bb" and payload.strings[3] == "", "strings")
    assert(#payload.big == 100000 and payload.big[100000] == 50000 and payload.big[3] == 1.5, "big")
    assert(#payload.nested.inner == 3 and payload.nested.inner[3] == 9, "nested")

    -- Top-level array, and mixed arrays are rejected instead of silently mis-typed
    expectedTaskId = TaskQueue:sendTask(workerId2, { 1, 2, 3 })
    taskId, payload = waitResult(workerId2, "top level array")
    assert(#payload == 3 and payload[3] == 3)
    assert(TaskQueue:sendTask(workerId2, { 1, "two", 3 }) == nil, "mixed arrays must be rejected")

    -- Error in the worker function: error result, worker keeps running
    expectedTaskId = TaskQueue:sendTask(workerId3, "error")
    taskId, payload, err = waitResult(workerId3, "error task")
    assert(taskId == expectedTaskId)
    assert(err ~= nil and err:find("intentional worker error"), "error message missing: " .. tostring(err))
    assert(err:find("traceback"), "traceback missing")

    -- nil result: empty result, worker keeps running
    expectedTaskId = TaskQueue:sendTask(workerId3, "nil")
    taskId, payload, err = waitResult(workerId3, "nil task")
    assert(taskId == expectedTaskId and payload == nil and err == nil, "nil result expected")

    expectedTaskId = TaskQueue:sendTask(workerId3, "still alive")
    taskId, payload = waitResult(workerId3, "task after errors")
    assert(taskId == expectedTaskId and payload == "still alive", "worker died after an error")

    TaskQueue:stopAllWorkers()

    Log.Info("WorkerTest:onInit: End (all checks passed)")
    self.done = true
end

function WorkerTest:onPreRender() end

function WorkerTest:onRender() end

function WorkerTest:onPostRender()
    if self.done then self:quit() end
end

return WorkerTest
