---@meta

-- List of the events used in the engine.
-- In Lua scripts it should be used as an event id.
-- To extend it in Lua, call `Event.AddEvents({"MyEvent1", "MyEvent2"})` function.
---@class Event
---@field PreSim integer Before physics update event
---@field Sim integer Physics update event
---@field PostSim integer After physics update event
---@field PreRender integer Before frame render event
---@field Render integer Frame render event
---@field PostRender integer After frame render event
---@field PreInput integer Before input handling event
---@field Input integer Input handling event
---@field PostInput integer After input handling event
---@field EngineEventsCount integer Specifies number of engine event types
Event = {
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
    EngineEventsCount = 9,
}

