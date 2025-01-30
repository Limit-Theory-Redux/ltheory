local libphx = require('libphx').lib
local PayloadConverter = require('Core.Util.PayloadConverter')

function onDef_TaskQueue_t(t, mt)
    mt.__index.startWorker = function(self, workerName, scriptPath, instancesCount)
        -- TODO: fix this if possible
        -- -@class WorkerId
        -- -@field workerName integer
        WorkerId.Register(workerName)

        local workerId = WorkerId[workerName]
        if not libphx.TaskQueue_StartWorker(self, workerId, workerName, scriptPath, instancesCount) then
            Log.Error("Cannot start worker: " .. workerName)
        end
        return workerId
    end

    mt.__index.sendTask = function(self, workerId, data)
        local payload = PayloadConverter:valueToPayload(data, true)
        -- Call ffi.gc here to avoid double free issue. See conversation here:
        -- https://discord.com/channels/695088786702336000/1265576869856542760/1280255882038607972
        local taskIdPtr = libphx.TaskQueue_SendTask(self, workerId, ffi.gc(payload, nil))
        local taskId = taskIdPtr[0]
        if taskId == nil then
            Log.Error("Cannot send task to " .. tostring(workerId))
        end
        return taskId
    end

    mt.__index.nextTaskResult = function(self, workerId)
        local taskResult = libphx.TaskQueue_NextTaskResult(self, workerId)
        if taskResult ~= nil then
            local payload = taskResult:payload()
            if payload ~= nil then
                local payloadValue = PayloadConverter:payloadToValue(payload)
                return taskResult:taskId(), payloadValue
            end
            return taskResult:taskId(), taskResult:error()
        end
        return nil, nil
    end
end
