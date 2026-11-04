-- AUTO GENERATED. DO NOT MODIFY!
-- CompareFn -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 CompareFn;
    ]]

    return 2, 'CompareFn'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CompareFn

    do -- C Definitions
        ffi.cdef [[
            cstr      CompareFn_ToString(CompareFn);
        ]]
    end

    do -- Global Symbol Table
        CompareFn = {
            Never        = 0,
            Less         = 1,
            Equal        = 2,
            LessEqual    = 3,
            Greater      = 4,
            NotEqual     = 5,
            GreaterEqual = 6,
            Always       = 7,

            ToString     = libphx.CompareFn_ToString,
        }

        if onDef_CompareFn then onDef_CompareFn(CompareFn, mt) end
        CompareFn = setmetatable(CompareFn, mt)
    end

    return CompareFn
end

return Loader
