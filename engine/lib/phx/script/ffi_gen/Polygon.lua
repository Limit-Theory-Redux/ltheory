-- AUTO GENERATED. DO NOT MODIFY!
-- Polygon ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Polygon {
            int32         vertices_size;
            int32         vertices_capacity;
            struct Vec3f* vertices_data;
        } Polygon;
    ]]

    return 1, 'Polygon'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Polygon

    do -- C Definitions
        ffi.cdef [[
            void   Polygon_Free        (Polygon*);
            Plane* Polygon_ToPlane     (Polygon const*);
            Plane* Polygon_ToPlaneFast (Polygon const*);
            void   Polygon_SplitSafe   (Polygon const*, Plane const* splitPlane, Polygon* back, Polygon* front);
            void   Polygon_Split       (Polygon*, Plane const* splitPlane, Polygon* back, Polygon* front);
            Vec3f  Polygon_GetCentroid (Polygon*);
            Error* Polygon_Validate    (Polygon*);
        ]]
    end

    do -- Global Symbol Table
        Polygon = {}

        if onDef_Polygon then onDef_Polygon(Polygon, mt) end
        Polygon = setmetatable(Polygon, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Polygon')
        local mt = {
            __index = {
                toPlane     = function(self)
                    local _instance = libphx.Polygon_ToPlane(self)
                    return Core.ManagedObject(_instance, libphx.Plane_Free)
                end,
                toPlaneFast = function(self)
                    local _instance = libphx.Polygon_ToPlaneFast(self)
                    return Core.ManagedObject(_instance, libphx.Plane_Free)
                end,
                splitSafe   = libphx.Polygon_SplitSafe,
                split       = libphx.Polygon_Split,
                getCentroid = libphx.Polygon_GetCentroid,
                validate    = function(self)
                    local _instance = libphx.Polygon_Validate(self)
                    return Core.ManagedObject(_instance, libphx.Error_Free)
                end,
            },
        }

        if onDef_Polygon_t then onDef_Polygon_t(t, mt) end
        Polygon_t = ffi.metatype(t, mt)
    end

    return Polygon
end

return Loader
