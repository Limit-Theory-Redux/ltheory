-- AUTO GENERATED. DO NOT MODIFY!
-- Intersect -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Intersect'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Intersect

    do -- C Definitions
        ffi.cdef [[
            bool Intersect_PointBox                 (Matrix* src, Matrix* dst);
            bool Intersect_PointTriangleBarycentric (Vec3f const* p, Triangle const* tri);
            bool Intersect_RayPlane                 (Ray const* ray, Plane const* plane, Position* pHit);
            bool Intersect_RayTriangleBarycentric   (Ray const* ray, Triangle const* tri, float tEpsilon, float* tHit);
            bool Intersect_RayTriangleMoller1       (Ray const* ray, Triangle const* tri, float* tHit);
            bool Intersect_RayTriangleMoller2       (Ray const* ray, Triangle const* tri, float* tHit);
            bool Intersect_LineSegmentPlane         (LineSegment const* lineSegment, Plane const* plane, Position* pHit);
            bool Intersect_RectRect                 (Vec4f const* a, Vec4f const* b);
            bool Intersect_RectRectFast             (Vec4f const* a, Vec4f const* b);
            bool Intersect_SphereTriangle           (Sphere const* sphere, Triangle const* triangle, Vec3f* pHit);
        ]]
    end

    do -- Global Symbol Table
        Intersect = {
            PointBox                 = libphx.Intersect_PointBox,
            PointTriangleBarycentric = libphx.Intersect_PointTriangleBarycentric,
            RayPlane                 = libphx.Intersect_RayPlane,
            RayTriangleBarycentric   = libphx.Intersect_RayTriangleBarycentric,
            RayTriangleMoller1       = libphx.Intersect_RayTriangleMoller1,
            RayTriangleMoller2       = libphx.Intersect_RayTriangleMoller2,
            LineSegmentPlane         = libphx.Intersect_LineSegmentPlane,
            RectRect                 = libphx.Intersect_RectRect,
            RectRectFast             = libphx.Intersect_RectRectFast,
            SphereTriangle           = libphx.Intersect_SphereTriangle,
        }

        if onDef_Intersect then onDef_Intersect(Intersect, mt) end
        Intersect = setmetatable(Intersect, mt)
    end

    return Intersect
end

return Loader
