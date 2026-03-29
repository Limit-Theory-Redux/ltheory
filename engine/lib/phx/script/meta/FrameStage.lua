-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Frame stages in order they are processed.
-- Standard game loop: gather input → simulate physics → render.
-- Events can be registered for each stage that will be dispatched in order they were sent.
---@class FrameStage
---@field PreInput integer Before input handling
---@field Input integer Input handling
---@field PostInput integer After input handling
---@field PreSim integer Before physics update
---@field Sim integer Physics update
---@field PostSim integer After physics update
---@field PreRender integer Before frame render
---@field Render integer Frame render
---@field PostRender integer After frame render
FrameStage = {
    -- Before input handling
    PreInput = 0,
    -- Input handling
    Input = 1,
    -- After input handling
    PostInput = 2,
    -- Before physics update
    PreSim = 3,
    -- Physics update
    Sim = 4,
    -- After physics update
    PostSim = 5,
    -- Before frame render
    PreRender = 6,
    -- Frame render
    Render = 7,
    -- After frame render
    PostRender = 8,
}

