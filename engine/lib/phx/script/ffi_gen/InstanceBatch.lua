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
            void           InstanceBatch_Free          (InstanceBatch*);
            InstanceBatch* InstanceBatch_Create        (Mesh* mesh, Renderer* r, CmdPrimitiveType* primitive);
            void           InstanceBatch_AddInstance   (InstanceBatch*, Matrix const* transform, float r, float g, float b, float a);
            void           InstanceBatch_Draw          (InstanceBatch*, Renderer* r);
            void           InstanceBatch_Clear         (InstanceBatch*);
            void           InstanceBatch_Flush         (InstanceBatch*, Renderer* r);
            int            InstanceBatch_InstanceCount (InstanceBatch const*);
        ]]
    end

    do -- Global Symbol Table
        InstanceBatch = {
            Create        = function(mesh, r, primitive)
                ffi.gc(primitive, nil)
                local _instance = libphx.InstanceBatch_Create(mesh, r, primitive)
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
                addInstance   = libphx.InstanceBatch_AddInstance,
                draw          = libphx.InstanceBatch_Draw,
                clear         = libphx.InstanceBatch_Clear,
                flush         = libphx.InstanceBatch_Flush,
                instanceCount = libphx.InstanceBatch_InstanceCount,
            },
        }

        if onDef_InstanceBatch_t then onDef_InstanceBatch_t(t, mt) end
        InstanceBatch_t = ffi.metatype(t, mt)
    end

    return InstanceBatch
end

return Loader
