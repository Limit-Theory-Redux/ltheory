-- AUTO GENERATED. DO NOT MODIFY!
-- Octree ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Octree {} Octree;
    ]]

    return 1, 'Octree'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Octree

    do -- C Definitions
        ffi.cdef [[
            void    Octree_Free         (Octree*);
            Octree* Octree_Create       (Box3f box0);
            Octree* Octree_FromMesh     (Mesh* mesh);
            double  Octree_GetAvgLoad   (Octree const*);
            int     Octree_GetMaxLoad   (Octree const*);
            uint64  Octree_GetMemory    (Octree const*);
            bool    Octree_IntersectRay (Octree const*, Matrix const* matrix, Vec3f const* ro, Vec3f const* rd);
            void    Octree_Add          (Octree*, Box3f box0, uint64 id);
            void    Octree_Draw         (Octree*);
        ]]
    end

    do -- Global Symbol Table
        Octree = {
            Create       = function(box0)
                local _instance = libphx.Octree_Create(box0)
                return Core.ManagedObject(_instance, libphx.Octree_Free)
            end,
            FromMesh     = function(mesh)
                local _instance = libphx.Octree_FromMesh(mesh)
                return Core.ManagedObject(_instance, libphx.Octree_Free)
            end,
        }

        if onDef_Octree then onDef_Octree(Octree, mt) end
        Octree = setmetatable(Octree, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Octree')
        local mt = {
            __index = {
                getAvgLoad   = libphx.Octree_GetAvgLoad,
                getMaxLoad   = libphx.Octree_GetMaxLoad,
                getMemory    = libphx.Octree_GetMemory,
                intersectRay = libphx.Octree_IntersectRay,
                add          = libphx.Octree_Add,
                draw         = libphx.Octree_Draw,
            },
        }

        if onDef_Octree_t then onDef_Octree_t(t, mt) end
        Octree_t = ffi.metatype(t, mt)
    end

    return Octree
end

return Loader
