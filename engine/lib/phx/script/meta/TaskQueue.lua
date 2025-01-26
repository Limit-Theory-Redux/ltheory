---@meta

-- Task queue is a worker threads manager.
-- It can be used to start either custom Lua scripts in a separate threads or predefined engine workers.
-- When started workers can accept tasks and return their results.
---@class TaskQueue
TaskQueue = {}

-- Start Lua worker with provided script file.
---@param workerId integer
---@param workerName string
---@param scriptPath string
---@param instancesCount integer
---@return boolean
function TaskQueue:startWorker(workerId, workerName, scriptPath, instancesCount) end

-- Stop Lua worker and remove it from the queue.
---@param workerId integer
---@return boolean
function TaskQueue:stopWorker(workerId) end

-- Stop all Lua workers and remove them from the queue.
function TaskQueue:stopAllWorkers() end

-- Returns number of tasks that were sent to the worker and whose results are not retrieved yet.
---@param workerId integer
---@return integer?
function TaskQueue:tasksInWork(workerId) end

-- Returns number of tasks waiting to be processed by the worker.
---@param workerId integer
---@return integer?
function TaskQueue:tasksWaiting(workerId) end

-- Returns number of tasks the worker is busy with.
---@param workerId integer
---@return integer?
function TaskQueue:tasksInProgress(workerId) end

-- Returns number of tasks finished by the worker and whose results can be retrieved.
---@param workerId integer
---@return integer?
function TaskQueue:tasksReady(workerId) end

-- Send a task to the Lua worker.
---@overload fun(workerId: integer, data: Payload|boolean|integer|number|string): integer?
function TaskQueue:sendTask(workerId, data) end

-- Returns next result of the finished worker task if any.
---@param workerId integer
---@return TaskResult?
function TaskQueue:nextTaskResult(workerId) end

-- Send a message to the echo worker.
---@param data string
---@return boolean
function TaskQueue:sendEcho(data) end

-- Get a response from the echo worker.
---@return string?
function TaskQueue:getEcho() end

