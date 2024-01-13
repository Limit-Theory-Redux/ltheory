-- LineSegment -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'LineSegment'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local LineSegment

    do -- C Definitions
        ffi.cdef [[
            void LineSegment_ToRay    (LineSegment const*, Ray* out);
            void LineSegment_FromRay  (Ray const* ray, LineSegment* out);
            cstr LineSegment_ToString (LineSegment const*);
        ]]
    end

    do -- Global Symbol Table
        LineSegment = {
            ToRay    = libphx.LineSegment_ToRay,
            FromRay  = libphx.LineSegment_FromRay,
            ToString = libphx.LineSegment_ToString,
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
                toRay    = libphx.LineSegment_ToRay,
                toString = libphx.LineSegment_ToString,
            },
        }

        if onDef_LineSegment_t then onDef_LineSegment_t(t, mt) end
        LineSegment_t = ffi.metatype(t, mt)
    end

    return LineSegment
end

return Loader
