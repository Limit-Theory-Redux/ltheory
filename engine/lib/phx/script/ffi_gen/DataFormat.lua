-- AUTO GENERATED. DO NOT MODIFY!
-- DataFormat ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 DataFormat;
    ]]

    return 2, 'DataFormat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local DataFormat

    do -- C Definitions
        ffi.cdef [[
            cstr       DataFormat_ToString(DataFormat);

            int DataFormat_GetSize (DataFormat this);
        ]]
    end

    do -- Global Symbol Table
        DataFormat = {
            I8       = 5120,
            U8       = 5121,
            I16      = 5122,
            U16      = 5123,
            I32      = 5124,
            U32      = 5125,
            Float    = 5126,

            ToString = libphx.DataFormat_ToString,

            GetSize = libphx.DataFormat_GetSize,
        }

        if onDef_DataFormat then onDef_DataFormat(DataFormat, mt) end
        DataFormat = setmetatable(DataFormat, mt)
    end

    return DataFormat
end

return Loader
