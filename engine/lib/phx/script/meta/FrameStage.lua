---@meta

-- Frame stages in order they are processed.
-- Events can be registered for each stage that will be dispatched in order they ere sent.
---@class FrameStage
---@field PreSim integer Before physics update
---@field Sim integer Physics update
---@field PostSim integer After physics update
---@field PreRender integer Before frame render
---@field Render integer Frame render
---@field PostRender integer After frame render
---@field PreInput integer Before input handling
---@field Input integer Input handling
---@field PostInput integer After input handling
FrameStage = {
    -- Before physics update
    PreSim = 0,
    -- Physics update
    Sim = 1,
    -- After physics update
    PostSim = 2,
    -- Before frame render
    PreRender = 3,
    -- Frame render
    Render = 4,
    -- After frame render
    PostRender = 5,
    -- Before input handling
    PreInput = 6,
    -- Input handling
    Input = 7,
    -- After input handling
    PostInput = 8,
}

