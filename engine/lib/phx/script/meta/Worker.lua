---@meta

-- Types of workers.
-- Can be extended on the Lua side.
---@enum Worker
Worker = {
    -- Example worker that replicates input value into the output
    Echo = 0,
    -- Specifies number of engine worker types
    EngineWorkersCount = 1,
}

