---@meta

-- List of the event types used in the engine.
-- In Lua scripts should be used as an event id.
-- To extend it in Lua scripts call `EventType.Register({"MyEventType1", "MyEventType2"})` function.
---@enum EventType
EventType = {
    ResourceLoadingResult = 0,
    EngineEventTypesCount = 1,
}

