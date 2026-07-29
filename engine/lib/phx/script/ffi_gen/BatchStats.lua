-- AUTO GENERATED. DO NOT MODIFY!
-- BatchStats ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BatchStats {} BatchStats;
    ]]

    return 1, 'BatchStats'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BatchStats

    do -- C Definitions
        ffi.cdef [[
            void   BatchStats_Free                 (BatchStats*);
            uint32 BatchStats_GetEntitiesSubmitted (BatchStats const*);
            uint32 BatchStats_GetEntitiesVisible   (BatchStats const*);
            uint32 BatchStats_GetEntitiesCulled    (BatchStats const*);
            uint32 BatchStats_GetTotalEntities     (BatchStats const*);
            uint32 BatchStats_GetCommandsGenerated (BatchStats const*);
            uint32 BatchStats_GetBatchesProcessed  (BatchStats const*);
        ]]
    end

    do -- Global Symbol Table
        BatchStats = {}

        if onDef_BatchStats then onDef_BatchStats(BatchStats, mt) end
        BatchStats = setmetatable(BatchStats, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BatchStats')
        local mt = {
            __index = {
                getEntitiesSubmitted = libphx.BatchStats_GetEntitiesSubmitted,
                getEntitiesVisible   = libphx.BatchStats_GetEntitiesVisible,
                getEntitiesCulled    = libphx.BatchStats_GetEntitiesCulled,
                getTotalEntities     = libphx.BatchStats_GetTotalEntities,
                getCommandsGenerated = libphx.BatchStats_GetCommandsGenerated,
                getBatchesProcessed  = libphx.BatchStats_GetBatchesProcessed,
            },
        }

        if onDef_BatchStats_t then onDef_BatchStats_t(t, mt) end
        BatchStats_t = ffi.metatype(t, mt)
    end

    return BatchStats
end

return Loader
