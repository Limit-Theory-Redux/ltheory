---@meta

-- Types of workers.
-- Can be extended on the Lua side.
---@class WorkerId
---@field Echo integer Example worker that replicates input value into the output
---@field EngineWorkersCount integer Specifies number of engine worker types
WorkerId = {
    -- Example worker that replicates input value into the output
    Echo = 0,
    -- Specifies number of engine worker types
    EngineWorkersCount = 1,
}

