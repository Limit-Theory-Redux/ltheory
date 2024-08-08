local libphx = require('libphx').lib

function onDef_EventType(t, mt)
    local EventType = t

    t.AddEventTypes = function(newTypes)
        if type(newTypes) ~= 'table' then
            Log.Error("new event types should be in a table")
            return
        end

        local nextFreeId = EventType.NextFreeId or EventType.EngineEventTypesCount

        for _, newType in ipairs(newTypes) do
            if EventType[newType] == nil then
                EventType[newType] = nextFreeId
                nextFreeId = nextFreeId + 1
            else
                Log.Error("event type '" .. newType .. "' already exists")
            end
        end

        EventType.NextFreeId = nextFreeId
    end
end
