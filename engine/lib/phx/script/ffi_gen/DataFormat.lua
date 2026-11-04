-- AUTO GENERATED. DO NOT MODIFY!
-- DataFormat ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 DataFormat;
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
            I8       = 0,
            U8       = 1,
            I16      = 2,
            U16      = 3,
            I32      = 4,
            U32      = 5,
            Float    = 6,

            ToString = libphx.DataFormat_ToString,

            GetSize = libphx.DataFormat_GetSize,
        }

        if onDef_DataFormat then onDef_DataFormat(DataFormat, mt) end
        DataFormat = setmetatable(DataFormat, mt)
    end

    return DataFormat
end

return Loader
