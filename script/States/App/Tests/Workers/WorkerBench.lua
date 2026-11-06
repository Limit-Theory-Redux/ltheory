local WorkerBench = require('States.Application')

function WorkerBench:onInit()
    Log.Info("WorkerBench:onInit: Start")

    local instancesCount = 4
    local messagesCount = 10000
    local pending = {}
    local left = 0

    local startTime = os.clock()
    Profiler.Enable()
    Profiler.Begin('WorkerBench')

    local workerId = TaskQueue:startWorker("TestWorker", "script/States/App/Tests/Workers/TestWorkerFunction.lua", instancesCount)
    assert(workerId ~= nil, "Cannot start bench worker")

    for _ = 1, messagesCount do
        local taskId = TaskQueue:sendTask(workerId, "TestPayload")
        pending[taskId] = true
        left = left + 1
    end

    Log.Debug("Messages sent: " .. left)

    local deadline = os.clock() + 120
    while left > 0 do
        assert(os.clock() < deadline, "WorkerBench timeout")
        local taskId, _ = TaskQueue:waitTaskResult(workerId, 100)
        if taskId ~= nil and pending[taskId] then
            pending[taskId] = nil
            left = left - 1
        end
    end

    local elapsed = os.clock() - startTime
    Log.Info("WorkerBench: %d tasks on %d instances in %.1f ms (%.1f us/task)", messagesCount, instancesCount,
        elapsed * 1000, elapsed * 1e6 / messagesCount)

    TaskQueue:stopAllWorkers()

    Profiler.End()
    Profiler.LoopMarker()
    Profiler.Disable()

    Log.Info("WorkerBench:onInit: End")
    self.done = true
end

function WorkerBench:onPreRender() end

function WorkerBench:onRender() end

function WorkerBench:onPostRender()
    if self.done then self:quit() end
end

return WorkerBench
