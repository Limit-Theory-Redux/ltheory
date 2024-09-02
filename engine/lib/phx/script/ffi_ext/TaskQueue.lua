local libphx = require('libphx').lib

function onDef_TaskQueue_t(t, mt)
    mt.__index.sendTask = function(self, workerId, data)
        local taskIdPtr = libphx.TaskQueue_SendTask(self, workerId, ffi.gc(PayloadConverter:valueToPayload(data, true), nil))
        return taskIdPtr[1]
    end

    mt.__index.nextTaskResult = function(self, workerId)
        local taskResult = libphx.TaskQueue_NextTaskResult(self, workerId)
        if taskResult ~= nil then
            local payloadValue = PayloadConverter:payloadToValue(taskResult:payload())

            return taskResult:taskId(), payloadValue
        end
        return nil, nil
    end
end
