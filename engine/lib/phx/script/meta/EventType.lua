---@meta

-- List of the event types used in the engine.
-- In Lua scripts should be used as an event id.
-- To extend it in Lua scripts call `EventType.AddEventTypes({"MyEventType1", "MyEventType2"})` function.
---@enum EventType
EventType = {
    PreSim = 0,
    Sim = 1,
    PostSim = 2,
    PreRender = 3,
    Render = 4,
    PostRender = 5,
    PreInput = 6,
    Input = 7,
    PostInput = 8,
    -- Specifies number of engine event types
    EngineEventTypesCount = 9,
}

