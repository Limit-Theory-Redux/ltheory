-- AUTO GENERATED. DO NOT MODIFY!
-- Plane -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Plane {
            float nx;
            float ny;
            float nz;
            float d;
        } Plane;
    ]]

    return 1, 'Plane'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Plane

    do -- C Definitions
        ffi.cdef [[
            void                   Plane_Free            (Plane*);
            PointClassification*   Plane_ClassifyPoint   (Plane const*, Vec3f const* p);
            PolygonClassification* Plane_ClassifyPolygon (Plane const*, Polygon const* polygon);
            Error*                 Plane_Validate        (Plane const*);
            Plane*                 Plane_FromPolygon     (Polygon const* polygon);
            Plane*                 Plane_FromPolygonFast (Polygon const* polygon);
        ]]
    end

    do -- Global Symbol Table
        Plane = {
            FromPolygon     = function(polygon)
                local _instance = libphx.Plane_FromPolygon(polygon)
                return Core.ManagedObject(_instance, libphx.Plane_Free)
            end,
            FromPolygonFast = function(polygon)
                local _instance = libphx.Plane_FromPolygonFast(polygon)
                return Core.ManagedObject(_instance, libphx.Plane_Free)
            end,
        }

        local mt = {
            __call = function(t, ...) return Plane_t(...) end,
        }

        if onDef_Plane then onDef_Plane(Plane, mt) end
        Plane = setmetatable(Plane, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Plane')
        local mt = {
            __index = {
                clone           = function(x) return Plane_t(x) end,
                classifyPoint   = function(self, p)
                    local _instance = libphx.Plane_ClassifyPoint(self, p)
                    return Core.ManagedObject(_instance, libphx.PointClassification_Free)
                end,
                classifyPolygon = function(self, polygon)
                    local _instance = libphx.Plane_ClassifyPolygon(self, polygon)
                    return Core.ManagedObject(_instance, libphx.PolygonClassification_Free)
                end,
                validate        = function(self)
                    local _instance = libphx.Plane_Validate(self)
                    return Core.ManagedObject(_instance, libphx.Error_Free)
                end,
            },
        }

        if onDef_Plane_t then onDef_Plane_t(t, mt) end
        Plane_t = ffi.metatype(t, mt)
    end

    return Plane
end

return Loader
