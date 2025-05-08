-- AUTO GENERATED. DO NOT MODIFY!
-- BoxTree ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BoxTree {} BoxTree;
    ]]

    return 1, 'BoxTree'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BoxTree

    do -- C Definitions
        ffi.cdef [[
            void     BoxTree_Free         (BoxTree*);
            BoxTree* BoxTree_Create       ();
            BoxTree* BoxTree_FromMesh     (Mesh const* mesh);
            void     BoxTree_Add          (BoxTree*, Box3f box3, uint8 const* data, uint64 data_size);
            uint64   BoxTree_GetMemory    (BoxTree const*);
            bool     BoxTree_IntersectRay (BoxTree const*, Matrix* matrix, Vec3f const* ro, Vec3f const* rd);
            void     BoxTree_Draw         (BoxTree const*, int maxDepth);
        ]]
    end

    do -- Global Symbol Table
        BoxTree = {
            Create       = function()
                local _instance = libphx.BoxTree_Create()
                return Core.ManagedObject(_instance, libphx.BoxTree_Free)
            end,
            FromMesh     = function(mesh)
                local _instance = libphx.BoxTree_FromMesh(mesh)
                return Core.ManagedObject(_instance, libphx.BoxTree_Free)
            end,
        }

        if onDef_BoxTree then onDef_BoxTree(BoxTree, mt) end
        BoxTree = setmetatable(BoxTree, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BoxTree')
        local mt = {
            __index = {
                add          = libphx.BoxTree_Add,
                getMemory    = libphx.BoxTree_GetMemory,
                intersectRay = libphx.BoxTree_IntersectRay,
                draw         = libphx.BoxTree_Draw,
            },
        }

        if onDef_BoxTree_t then onDef_BoxTree_t(t, mt) end
        BoxTree_t = ffi.metatype(t, mt)
    end

    return BoxTree
end

return Loader
