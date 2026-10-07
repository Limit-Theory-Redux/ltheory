-- AUTO GENERATED. DO NOT MODIFY!
-- InstanceField ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InstanceField {} InstanceField;
    ]]

    return 1, 'InstanceField'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InstanceField

    do -- C Definitions
        ffi.cdef [[
            void           InstanceField_Free           (InstanceField*);
            InstanceField* InstanceField_Create         (float const* pos, uint64 pos_size, float const* scales, uint64 scales_size, uint32 const* chunkOffsets, uint64 chunkOffsets_size, uint32 const* chunkIndices, uint64 chunkIndices_size, float const* chunkCentroids, uint64 chunkCentroids_size);
            void           InstanceField_SetWorkers     (InstanceField*, uint32 workers);
            void           InstanceField_SetLodCount    (InstanceField*, uint32 count);
            uint32         InstanceField_Cull           (InstanceField*, double eyeX, double eyeY, double eyeZ, double fwdX, double fwdY, double fwdZ, double originX, double originY, double originZ, double pxPerUnitSq, double renderDistSq, uint32 maxDrawn, uint32 const* spawned, uint64 spawned_size);
            uint32         InstanceField_GetCount       (InstanceField const*, uint32 lod);
            uint32         InstanceField_GetLodOrderLen (InstanceField const*);
            uint32         InstanceField_GetLodOrder    (InstanceField const*, uint32 k);
            void           InstanceField_Draw           (InstanceField const*, RenderPass const* pass, Renderer* r, uint32 lod, Mesh* mesh);
        ]]
    end

    do -- Global Symbol Table
        InstanceField = {
            Create         = function(pos, scales, chunkOffsets, chunkIndices, chunkCentroids)
                local _instance = libphx.InstanceField_Create(pos, scales, chunkOffsets, chunkIndices, chunkCentroids)
                return Core.ManagedObject(_instance, libphx.InstanceField_Free)
            end,
        }

        if onDef_InstanceField then onDef_InstanceField(InstanceField, mt) end
        InstanceField = setmetatable(InstanceField, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InstanceField')
        local mt = {
            __index = {
                setWorkers     = libphx.InstanceField_SetWorkers,
                setLodCount    = libphx.InstanceField_SetLodCount,
                cull           = libphx.InstanceField_Cull,
                getCount       = libphx.InstanceField_GetCount,
                getLodOrderLen = libphx.InstanceField_GetLodOrderLen,
                getLodOrder    = libphx.InstanceField_GetLodOrder,
                draw           = libphx.InstanceField_Draw,
            },
        }

        if onDef_InstanceField_t then onDef_InstanceField_t(t, mt) end
        InstanceField_t = ffi.metatype(t, mt)
    end

    return InstanceField
end

return Loader
