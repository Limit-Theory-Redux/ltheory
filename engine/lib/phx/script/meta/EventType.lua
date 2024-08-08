---@meta

-- List of the event types used in the engine.
-- In Lua scripts should be used as an event id.
-- To extend it in Lua scripts call `EventType.AddEventTypes({"MyEventType1", "MyEventType2"})` function.
---@enum EventType
EventType = {
    -- Before physics update event
    PreSim = 0,
    -- Physics update event
    Sim = 1,
    -- After physics update event
    PostSim = 2,
    -- Before frame render event
    PreRender = 3,
    -- Frame render event
    Render = 4,
    -- After frame render event
    PostRender = 5,
    -- Before input handling event
    PreInput = 6,
    -- Input handling event
    Input = 7,
    -- After input handling event
    PostInput = 8,
    -- Specifies number of engine event types
    EngineEventTypesCount = 9,
}

