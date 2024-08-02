---@meta

-- List of the event types used in the engine.
-- In Lua scripts should be used as an event id.
-- To extend it in Lua scripts call `EventType.AddEventTypes({"MyEventType1", "MyEventType2"})` function.
---@enum EventType
EventType = {
    ResourceLoadingResult = 0,
    -- Specifies number of engine event types
    EngineEventTypesCount = 1,
}

