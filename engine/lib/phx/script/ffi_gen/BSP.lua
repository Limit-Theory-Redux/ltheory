-- AUTO GENERATED. DO NOT MODIFY!
-- BSP -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BSP {} BSP;
    ]]

    return 1, 'BSP'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BSP

    do -- C Definitions
        ffi.cdef [[
            void        BSP_Free                        (BSP*);
            BSP*        BSP_Create                      (Mesh const* mesh);
            bool        BSP_IntersectRay                (BSP const*, Ray* ray, float* tHit);
            bool        BSP_IntersectLineSegment        (BSP const*, LineSegment const* lineSegment, Vec3f* pHit);
            bool        BSP_IntersectSphere             (BSP const*, Sphere const* sphere, Vec3f* pHit);
            BSPNodeRef* BSP_GetNode                     (BSP const*, BSPNodeRef* nodeRef, BSPNodeRel relationship);
            void        BSP_DrawNode                    (BSP*, BSPNodeRef* nodeRef, Color const* color);
            void        BSP_DrawNodeSplit               (BSP*, BSPNodeRef* nodeRef);
            void        BSP_DrawLineSegment             (BSP*, LineSegment const* lineSegment, Position const* eye);
            void        BSP_DrawSphere                  (BSP*, Sphere const* sphere);
            void        BSP_PrintRayProfilingData       (BSP const*, double totalTime);
            void        BSP_PrintSphereProfilingData    (BSP const*, double totalTime);
            bool        BSP_GetIntersectSphereTriangles (BSP const*, Sphere const* sphere, IntersectSphereProfiling* sphereProf);
            BSPNodeRef* BSP_GetLeaf                     (BSP const*, int leafIndex);
        ]]
    end

    do -- Global Symbol Table
        BSP = {
            Create                      = function(mesh)
                local _instance = libphx.BSP_Create(mesh)
                return Core.ManagedObject(_instance, libphx.BSP_Free)
            end,
        }

        if onDef_BSP then onDef_BSP(BSP, mt) end
        BSP = setmetatable(BSP, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BSP')
        local mt = {
            __index = {
                intersectRay                = libphx.BSP_IntersectRay,
                intersectLineSegment        = libphx.BSP_IntersectLineSegment,
                intersectSphere             = libphx.BSP_IntersectSphere,
                getNode                     = function(self, nodeRef, relationship)
                    ffi.gc(nodeRef, nil)
                    local _instance = libphx.BSP_GetNode(self, nodeRef, relationship)
                    return Core.ManagedObject(_instance, libphx.BSPNodeRef_Free)
                end,
                drawNode                    = function(self, nodeRef, color)
                    ffi.gc(nodeRef, nil)
                    libphx.BSP_DrawNode(self, nodeRef, color)
                end,
                drawNodeSplit               = function(self, nodeRef)
                    ffi.gc(nodeRef, nil)
                    libphx.BSP_DrawNodeSplit(self, nodeRef)
                end,
                drawLineSegment             = libphx.BSP_DrawLineSegment,
                drawSphere                  = libphx.BSP_DrawSphere,
                printRayProfilingData       = libphx.BSP_PrintRayProfilingData,
                printSphereProfilingData    = libphx.BSP_PrintSphereProfilingData,
                getIntersectSphereTriangles = libphx.BSP_GetIntersectSphereTriangles,
                getLeaf                     = function(self, leafIndex)
                    local _instance = libphx.BSP_GetLeaf(self, leafIndex)
                    return Core.ManagedObject(_instance, libphx.BSPNodeRef_Free)
                end,
            },
        }

        if onDef_BSP_t then onDef_BSP_t(t, mt) end
        BSP_t = ffi.metatype(t, mt)
    end

    return BSP
end

return Loader
