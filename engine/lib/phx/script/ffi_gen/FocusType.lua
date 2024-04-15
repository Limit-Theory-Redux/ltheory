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
            FocusType FocusType_Mouse;
            FocusType FocusType_Scroll;

            cstr      FocusType_ToString(FocusType);
        ]]
    end

    do -- Global Symbol Table
        FocusType = {
            Mouse    = libphx.FocusType_Mouse,
            Scroll   = libphx.FocusType_Scroll,

            ToString = libphx.FocusType_ToString,
        }

        if onDef_FocusType then onDef_FocusType(FocusType, mt) end
        FocusType = setmetatable(FocusType, mt)
    end

    return FocusType
end

return Loader
