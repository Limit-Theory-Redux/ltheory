---@meta

---@class TaskQueue
TaskQueue = {}

---@param workerId integer
---@param workerName string
---@param scriptPath string
---@return boolean
function TaskQueue:startWorker(workerId, workerName, scriptPath) end

---@param workerId integer
---@return boolean
function TaskQueue:stopWorker(workerId) end

---@param workerId integer
---@return integer?
function TaskQueue:tasksInProgress(workerId) end

---@param workerId integer
---@param data Payload
---@return integer?
function TaskQueue:sendTask(workerId, data) end

---@param workerId integer
---@return TaskResult?
function TaskQueue:nextTaskResult(workerId) end

---@param data string
---@return boolean
function TaskQueue:sendEcho(data) end

---@return string?
function TaskQueue:getEcho() end

