function onDef_WorkerId(t, mt)
    local WorkerId = t

    -- Add new worker types to the Worker table.
    -- Fail if worker already exists.
    ---@param newWorkers table List of names of new workers
    t.AddWorkers = function(newWorkers)
        if type(newWorkers) ~= 'table' then
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
