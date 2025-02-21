-- AUTO GENERATED. DO NOT MODIFY!
-- LineSegment -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct LineSegment {
            double p0x;
            double p0y;
            double p0z;
            double p1x;
            double p1y;
            double p1z;
        } LineSegment;
    ]]

    return 1, 'LineSegment'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local LineSegment

    do -- C Definitions
        ffi.cdef [[
            void         LineSegment_Free     (LineSegment*);
            Ray*         LineSegment_ToRay    (LineSegment const*);
            LineSegment* LineSegment_FromRay  (Ray const* ray);
            cstr         LineSegment_ToString (LineSegment const*);
        ]]
    end

    do -- Global Symbol Table
        LineSegment = {
            FromRay  = function(ray)
                local _instance = libphx.LineSegment_FromRay(ray)
                return Core.ManagedObject(_instance, libphx.LineSegment_Free)
            end,
        }

        local mt = {
            __call = function(t, ...) return LineSegment_t(...) end,
        }

        if onDef_LineSegment then onDef_LineSegment(LineSegment, mt) end
        LineSegment = setmetatable(LineSegment, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('LineSegment')
        local mt = {
            __tostring = function(self) return ffi.string(libphx.LineSegment_ToString(self)) end,
            __index = {
                clone    = function(x) return LineSegment_t(x) end,
                toRay    = function(self)
                    local _instance = libphx.LineSegment_ToRay(self)
                    return Core.ManagedObject(_instance, libphx.Ray_Free)
                end,
                toString = libphx.LineSegment_ToString,
            },
        }

        if onDef_LineSegment_t then onDef_LineSegment_t(t, mt) end
        LineSegment_t = ffi.metatype(t, mt)
    end

    return LineSegment
end

return Loader
