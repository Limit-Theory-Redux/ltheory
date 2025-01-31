function onDef_WorkerId(t, mt)
    local WorkerId = t

    ---@class WorkerId
    ---@field Register fun(workers: table<string>) Register a new worker types to the WorkerId table. Fail if worker already exists.
    t.Register = function(newWorkers)
        if type(newWorkers) == 'string' then
            newWorkers = { newWorkers }
        elseif type(newWorkers) ~= 'table' then
            Log.Error("new workers should be in a table")
            return
        end

        local nextFreeId = WorkerId.NextFreeId or WorkerId.EngineWorkersCount

        for _, newWorker in ipairs(newWorkers) do
            if WorkerId[newWorker] == nil then
                WorkerId[newWorker] = nextFreeId
                nextFreeId = nextFreeId + 1
            else
                Log.Error("worker '" .. newWorker .. "' already exists")
            end
        end

        WorkerId.NextFreeId = nextFreeId
    end
end
