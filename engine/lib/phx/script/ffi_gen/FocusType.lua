-- AUTO GENERATED. DO NOT MODIFY!
-- FocusType -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 FocusType;
    ]]

    return 2, 'FocusType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local FocusType

    do -- C Definitions
        ffi.cdef [[
            cstr      FocusType_ToString(FocusType);
        ]]
    end

    do -- Global Symbol Table
        FocusType = {
            Mouse    = 0,
            Scroll   = 1,

            ToString = libphx.FocusType_ToString,
        }

        if onDef_FocusType then onDef_FocusType(FocusType, mt) end
        FocusType = setmetatable(FocusType, mt)
    end

    return FocusType
end

return Loader
