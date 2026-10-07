-- AUTO GENERATED. DO NOT MODIFY!
-- PolygonMode -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 PolygonMode;
    ]]

    return 2, 'PolygonMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PolygonMode

    do -- C Definitions
        ffi.cdef [[
            cstr        PolygonMode_ToString(PolygonMode);
        ]]
    end

    do -- Global Symbol Table
        PolygonMode = {
            Fill     = 0,
            Line     = 1,

            ToString = libphx.PolygonMode_ToString,
        }

        if onDef_PolygonMode then onDef_PolygonMode(PolygonMode, mt) end
        PolygonMode = setmetatable(PolygonMode, mt)
    end

    return PolygonMode
end

return Loader
