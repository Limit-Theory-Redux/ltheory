---@meta

-- Frame stages in order they are processed.
-- Events can be registered for each stage that will be dispatched in order they ere sent.
---@enum FrameStage
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

