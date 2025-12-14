function onDef_Event(t, mt)
    local Event = t

    ---@class Event
    ---@field AddEvents fun(newEvents: table)

    -- Add new events to the Event table.
    -- Fail if event already exists.
    ---@param newEvents table List of names of new events
    t.AddEvents = function(newEvents)
        if type(newEvents) ~= 'table' then
            Log.Error("new events should be in a table")
            return
        end

        local nextFreeId = Event.NextFreeId or Event.EngineEventsCount

        for _, newEvent in ipairs(newEvents) do
            if Event[newEvent] == nil then
                Event[newEvent] = nextFreeId
                nextFreeId = nextFreeId + 1
            else
                Log.Error("event '" .. newEvent .. "' already exists")
            end
        end

        Event.NextFreeId = nextFreeId
    end
end
