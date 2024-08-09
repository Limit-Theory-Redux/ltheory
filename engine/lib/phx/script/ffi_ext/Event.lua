local libphx = require('libphx').lib

function onDef_Event(t, mt)
    local Event = t

    t.AddEvents = function(newTypes)
        if type(newTypes) ~= 'table' then
            Log.Error("new events should be in a table")
            return
        end

        local nextFreeId = Event.NextFreeId or Event.EngineEventsCount

        for _, newType in ipairs(newTypes) do
            if Event[newType] == nil then
                Event[newType] = nextFreeId
                nextFreeId = nextFreeId + 1
            else
                Log.Error("event '" .. newType .. "' already exists")
            end
        end

        Event.NextFreeId = nextFreeId
    end
end
