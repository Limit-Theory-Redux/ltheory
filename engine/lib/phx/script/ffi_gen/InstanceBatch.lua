-- AUTO GENERATED. DO NOT MODIFY!
-- InstanceBatch ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InstanceBatch {} InstanceBatch;
    ]]

    return 1, 'InstanceBatch'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InstanceBatch

    do -- C Definitions
        ffi.cdef [[
            void           InstanceBatch_Free              (InstanceBatch*);
            InstanceBatch* InstanceBatch_Create            (Mesh* mesh, int primitive);
            void           InstanceBatch_AddInstance       (InstanceBatch*, float m00, float m01, float m02, float m03, float m10, float m11, float m12, float m13, float m20, float m21, float m22, float m23, float m30, float m31, float m32, float m33, float r, float g, float b, float a);
            void           InstanceBatch_AddInstanceMatrix (InstanceBatch*, Matrix const* matrix, float r, float g, float b, float a);
            int            InstanceBatch_GetInstanceCount  (InstanceBatch const*);
            void           InstanceBatch_Clear             (InstanceBatch*);
            void           InstanceBatch_Draw              (InstanceBatch const*);
            void           InstanceBatch_Flush             (InstanceBatch*);
        ]]
    end

    do -- Global Symbol Table
        InstanceBatch = {
            Create            = function(mesh, primitive)
                local _instance = libphx.InstanceBatch_Create(mesh, primitive)
                return Core.ManagedObject(_instance, libphx.InstanceBatch_Free)
            end,
        }

        if onDef_InstanceBatch then onDef_InstanceBatch(InstanceBatch, mt) end
        InstanceBatch = setmetatable(InstanceBatch, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InstanceBatch')
        local mt = {
            __index = {
                addInstance       = libphx.InstanceBatch_AddInstance,
                addInstanceMatrix = libphx.InstanceBatch_AddInstanceMatrix,
                getInstanceCount  = libphx.InstanceBatch_GetInstanceCount,
                clear             = libphx.InstanceBatch_Clear,
                draw              = libphx.InstanceBatch_Draw,
                flush             = libphx.InstanceBatch_Flush,
            },
        }

        if onDef_InstanceBatch_t then onDef_InstanceBatch_t(t, mt) end
        InstanceBatch_t = ffi.metatype(t, mt)
    end

    return InstanceBatch
end

return Loader
