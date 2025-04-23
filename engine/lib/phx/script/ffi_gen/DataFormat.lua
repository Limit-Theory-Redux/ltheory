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
            DataFormat DataFormat_I8;
            DataFormat DataFormat_U8;
            DataFormat DataFormat_I16;
            DataFormat DataFormat_U16;
            DataFormat DataFormat_I32;
            DataFormat DataFormat_U32;
            DataFormat DataFormat_Float;

            cstr       DataFormat_ToString(DataFormat);

            int DataFormat_GetSize (DataFormat this);
        ]]
    end

    do -- Global Symbol Table
        DataFormat = {
            I8       = libphx.DataFormat_I8,
            U8       = libphx.DataFormat_U8,
            I16      = libphx.DataFormat_I16,
            U16      = libphx.DataFormat_U16,
            I32      = libphx.DataFormat_I32,
            U32      = libphx.DataFormat_U32,
            Float    = libphx.DataFormat_Float,

            ToString = libphx.DataFormat_ToString,

            GetSize = libphx.DataFormat_GetSize,
        }

        if onDef_DataFormat then onDef_DataFormat(DataFormat, mt) end
        DataFormat = setmetatable(DataFormat, mt)
    end

    return DataFormat
end

return Loader
