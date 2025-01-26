-- AUTO GENERATED. DO NOT MODIFY!
-- TaskResult ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TaskResult {} TaskResult;
    ]]

    return 1, 'TaskResult'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TaskResult

    do -- C Definitions
        ffi.cdef [[
            void           TaskResult_Free     (TaskResult*);
            uint16         TaskResult_WorkerId (TaskResult const*);
            uint64         TaskResult_TaskId   (TaskResult const*);
            Payload const* TaskResult_Payload  (TaskResult const*);
            cstr           TaskResult_Error    (TaskResult const*);
        ]]
    end

    do -- Global Symbol Table
        TaskResult = {}

        if onDef_TaskResult then onDef_TaskResult(TaskResult, mt) end
        TaskResult = setmetatable(TaskResult, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TaskResult')
        local mt = {
            __index = {
                workerId = libphx.TaskResult_WorkerId,
                taskId   = libphx.TaskResult_TaskId,
                payload  = libphx.TaskResult_Payload,
                error    = libphx.TaskResult_Error,
            },
        }

        if onDef_TaskResult_t then onDef_TaskResult_t(t, mt) end
        TaskResult_t = ffi.metatype(t, mt)
    end

    return TaskResult
end

return Loader
