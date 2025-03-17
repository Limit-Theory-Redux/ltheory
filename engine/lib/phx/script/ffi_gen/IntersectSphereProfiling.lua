-- AUTO GENERATED. DO NOT MODIFY!
-- IntersectSphereProfiling ----------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct IntersectSphereProfiling {
            int32                nodes;
            int32                leaves;
            int32                triangles;
            int32                triangleTests_size;
            int32                triangleTests_capacity;
            struct TriangleTest* triangleTests_data;
        } IntersectSphereProfiling;
    ]]

    return 1, 'IntersectSphereProfiling'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local IntersectSphereProfiling

    do -- C Definitions
        ffi.cdef [[
            void                      IntersectSphereProfiling_Free   (IntersectSphereProfiling*);
            IntersectSphereProfiling* IntersectSphereProfiling_Create ();
        ]]
    end

    do -- Global Symbol Table
        IntersectSphereProfiling = {
            Create = function()
                local _instance = libphx.IntersectSphereProfiling_Create()
                return Core.ManagedObject(_instance, libphx.IntersectSphereProfiling_Free)
            end,
        }

        if onDef_IntersectSphereProfiling then onDef_IntersectSphereProfiling(IntersectSphereProfiling, mt) end
        IntersectSphereProfiling = setmetatable(IntersectSphereProfiling, mt)
    end

    return IntersectSphereProfiling
end

return Loader
