local libphx = require('libphx').lib

function onDef_Worker(t, mt)
    local Worker = t

    -- Add new worker types to the Worker table.
    -- Fail if worker already exists.
    ---@param newWorkers table List of names of new workers
    t.AddWorkers = function(newWorkers)
        if type(newWorkers) ~= 'table' then
            Log.Error("new workers should be in a table")
            return
        end

        local nextFreeId = Worker.NextFreeId or Worker.EngineWorkersCount

        for _, newWorker in ipairs(newWorkers) do
            if Worker[newWorker] == nil then
                Worker[newWorker] = nextFreeId
                nextFreeId = nextFreeId + 1
            else
                Log.Error("worker '" .. newWorker .. "' already exists")
            end
        end

        Worker.NextFreeId = nextFreeId
    end
end
