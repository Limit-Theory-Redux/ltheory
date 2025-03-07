-- AUTO GENERATED. DO NOT MODIFY!
-- Triangle --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Triangle {
            Vec3f vertices[3];
        } Triangle;
    ]]

    return 1, 'Triangle'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Triangle

    do -- C Definitions
        ffi.cdef [[
            void   Triangle_Free        (Triangle*);
            Plane* Triangle_ToPlane     (Triangle const*);
            Plane* Triangle_ToPlaneFast (Triangle const*);
            float  Triangle_GetArea     (Triangle const*);
            Error* Triangle_Validate    (Triangle const*);
        ]]
    end

    do -- Global Symbol Table
        Triangle = {}

        local mt = {
            __call = function(t, ...) return Triangle_t(...) end,
        }

        if onDef_Triangle then onDef_Triangle(Triangle, mt) end
        Triangle = setmetatable(Triangle, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Triangle')
        local mt = {
            __index = {
                clone       = function(x) return Triangle_t(x) end,
                toPlane     = function(self)
                    local _instance = libphx.Triangle_ToPlane(self)
                    return Core.ManagedObject(_instance, libphx.Plane_Free)
                end,
                toPlaneFast = function(self)
                    local _instance = libphx.Triangle_ToPlaneFast(self)
                    return Core.ManagedObject(_instance, libphx.Plane_Free)
                end,
                getArea     = libphx.Triangle_GetArea,
                validate    = function(self)
                    local _instance = libphx.Triangle_Validate(self)
                    return Core.ManagedObject(_instance, libphx.Error_Free)
                end,
            },
        }

        if onDef_Triangle_t then onDef_Triangle_t(t, mt) end
        Triangle_t = ffi.metatype(t, mt)
    end

    return Triangle
end

return Loader
