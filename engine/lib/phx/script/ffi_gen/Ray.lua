-- AUTO GENERATED. DO NOT MODIFY!
-- Ray -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Ray {
            double px;
            double py;
            double pz;
            double dirx;
            double diry;
            double dirz;
            double tMin;
            double tMax;
        } Ray;
    ]]

    return 1, 'Ray'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Ray

    do -- C Definitions
        ffi.cdef [[
            void         Ray_Free                         (Ray*);
            Position     Ray_GetPoint                     (Ray const*, double t);
            bool         Ray_IntersectPlane               (Ray const*, Plane const* plane, Position* pHit);
            bool         Ray_IntersectTriangleBarycentric (Ray const*, Triangle const* tri, float tEpsilon, float* tHit);
            bool         Ray_IntersectTriangleMoller1     (Ray const*, Triangle const* tri, float* tHit);
            bool         Ray_IntersectTriangleMoller2     (Ray const*, Triangle const* tri, float* tHit);
            LineSegment* Ray_ToLineSegment                (Ray const*);
            Ray*         Ray_FromLineSegment              (LineSegment const* lineSegment);
        ]]
    end

    do -- Global Symbol Table
        Ray = {
            FromLineSegment              = function(lineSegment)
                local _instance = libphx.Ray_FromLineSegment(lineSegment)
                return Core.ManagedObject(_instance, libphx.Ray_Free)
            end,
        }

        local mt = {
            __call = function(t, ...) return Ray_t(...) end,
        }

        if onDef_Ray then onDef_Ray(Ray, mt) end
        Ray = setmetatable(Ray, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Ray')
        local mt = {
            __index = {
                clone                        = function(x) return Ray_t(x) end,
                getPoint                     = libphx.Ray_GetPoint,
                intersectPlane               = libphx.Ray_IntersectPlane,
                intersectTriangleBarycentric = libphx.Ray_IntersectTriangleBarycentric,
                intersectTriangleMoller1     = libphx.Ray_IntersectTriangleMoller1,
                intersectTriangleMoller2     = libphx.Ray_IntersectTriangleMoller2,
                toLineSegment                = function(self)
                    local _instance = libphx.Ray_ToLineSegment(self)
                    return Core.ManagedObject(_instance, libphx.LineSegment_Free)
                end,
            },
        }

        if onDef_Ray_t then onDef_Ray_t(t, mt) end
        Ray_t = ffi.metatype(t, mt)
    end

    return Ray
end

return Loader
