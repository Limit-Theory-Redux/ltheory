-- TaskQueue -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TaskQueue {} TaskQueue;
    ]]

    return 1, 'TaskQueue'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TaskQueue

    do -- C Definitions
        ffi.cdef [[
            void          TaskQueue_Free            (TaskQueue*);
            bool          TaskQueue_StartWorker     (TaskQueue*, uint16 workerId, cstr workerName, cstr scriptPath, uint64 instancesCount);
            bool          TaskQueue_StopWorker      (TaskQueue*, uint16 workerId);
            void          TaskQueue_StopAllWorkers  (TaskQueue*);
            uint64 const* TaskQueue_TasksInWork     (TaskQueue const*, uint16 workerId);
            uint64 const* TaskQueue_TasksWaiting    (TaskQueue const*, uint16 workerId);
            uint64 const* TaskQueue_TasksInProgress (TaskQueue const*, uint16 workerId);
            uint64 const* TaskQueue_TasksReady      (TaskQueue const*, uint16 workerId);
            uint64 const* TaskQueue_SendTask        (TaskQueue*, uint16 workerId, Payload* data);
            TaskResult*   TaskQueue_NextTaskResult  (TaskQueue*, uint16 workerId);
            bool          TaskQueue_SendEcho        (TaskQueue*, cstr data);
            cstr          TaskQueue_GetEcho         (TaskQueue*);
        ]]
    end

    do -- Global Symbol Table
        TaskQueue = {}

        if onDef_TaskQueue then onDef_TaskQueue(TaskQueue, mt) end
        TaskQueue = setmetatable(TaskQueue, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TaskQueue')
        local mt = {
            __index = {
                startWorker     = libphx.TaskQueue_StartWorker,
                stopWorker      = libphx.TaskQueue_StopWorker,
                stopAllWorkers  = libphx.TaskQueue_StopAllWorkers,
                tasksInWork     = libphx.TaskQueue_TasksInWork,
                tasksWaiting    = libphx.TaskQueue_TasksWaiting,
                tasksInProgress = libphx.TaskQueue_TasksInProgress,
                tasksReady      = libphx.TaskQueue_TasksReady,
                sendTask        = libphx.TaskQueue_SendTask,
                nextTaskResult  = function(...)
                    local instance = libphx.TaskQueue_NextTaskResult(...)
                    return Core.ManagedObject(instance, libphx.TaskResult_Free)
                end,
                sendEcho        = libphx.TaskQueue_SendEcho,
                getEcho         = libphx.TaskQueue_GetEcho,
            },
        }

        if onDef_TaskQueue_t then onDef_TaskQueue_t(t, mt) end
        TaskQueue_t = ffi.metatype(t, mt)
    end

    return TaskQueue
end

return Loader
