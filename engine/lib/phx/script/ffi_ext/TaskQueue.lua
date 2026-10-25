local libphx = require('libphx').lib
local PayloadConverter = require('Core.Util.PayloadConverter')

-- Convert a TaskResult pointer into (taskId, value, errorMessage).
local function unpackTaskResult(taskResult)
    if taskResult == nil then
        return nil, nil, nil
    end
    local taskId = tonumber(taskResult:taskId())
    local payload = taskResult:payload()
    if payload ~= nil then
        return taskId, PayloadConverter:payloadToValue(payload), nil
    end
    local err = taskResult:error()
    if err ~= nil then
        return taskId, nil, ffi.string(err)
    end
    -- worker function returned nil
    return taskId, nil, nil
end

function onDef_TaskQueue_t(t, mt)
    ---@param workerName string
    ---@param scriptPath string
    ---@param instancesCount integer
    ---@return integer? workerId nil if the worker could not be started
    mt.__index.startWorker = function(self, workerName, scriptPath, instancesCount)
        -- TODO: fix this if possible
        -- -@class WorkerId
        -- -@field workerName integer
        if WorkerId[workerName] == nil then
            WorkerId.Register(workerName)
        end

        local workerId = WorkerId[workerName]
        if not libphx.TaskQueue_StartWorker(self, workerId, workerName, scriptPath, instancesCount) then
            Log.Warn("Cannot start worker '" .. tostring(workerName) .. "' (script: " .. tostring(scriptPath) ..
                "). See the log above for the reason.")
            return nil
        end
        return workerId
    end

    ---@return integer? taskId nil if the task could not be sent
    mt.__index.sendTask = function(self, workerId, data)
        if workerId == nil then
            Log.Warn("Cannot send task: worker id is nil (was the worker started successfully?)")
            return nil
        end
        local payload = PayloadConverter:valueToPayload(data, true)
        if payload == nil then
            Log.Warn("Cannot send task to worker " .. tostring(workerId) .. ": unsupported or nil task data")
            return nil
        end
        -- Call ffi.gc here to avoid double free issue. See conversation here:
        -- https://discord.com/channels/695088786702336000/1265576869856542760/1280255882038607972
        local taskIdPtr = libphx.TaskQueue_SendTask(self, workerId, ffi.gc(payload, nil))
        if taskIdPtr == nil then
            Log.Warn("Cannot send task to worker " .. tostring(workerId))
            return nil
        end
        return tonumber(taskIdPtr[0])
    end

    -- Non-blocking: returns nothing when no result is ready.
    ---@return integer? taskId
    ---@return any? value task result, or the error message string when the task failed (see errorMessage)
    ---@return string? errorMessage set when the worker function raised an error
    mt.__index.nextTaskResult = function(self, workerId)
        local taskId, value, err = unpackTaskResult(libphx.TaskQueue_NextTaskResult(self, workerId))
        if err ~= nil then
            return taskId, err, err
        end
        return taskId, value, nil
    end

    -- Like nextTaskResult but waits up to timeoutMs for a result.
    mt.__index.waitTaskResult = function(self, workerId, timeoutMs)
        local taskId, value, err = unpackTaskResult(libphx.TaskQueue_WaitTaskResult(self, workerId, timeoutMs or 0))
        if err ~= nil then
            return taskId, err, err
        end
        return taskId, value, nil
    end
end
