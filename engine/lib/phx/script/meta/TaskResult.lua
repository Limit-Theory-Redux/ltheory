---@meta

-- Task result information.
-- Result data can be either payload on success or error message on fail.
---@class TaskResult
TaskResult = {}

---@return integer
function TaskResult:workerId() end

---@return integer
function TaskResult:taskId() end

---@return Payload?
function TaskResult:payload() end

---@return string?
function TaskResult:error() end

