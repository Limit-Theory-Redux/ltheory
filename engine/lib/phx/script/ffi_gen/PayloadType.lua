-- AUTO GENERATED. DO NOT MODIFY!
-- PayloadType -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 PayloadType;
    ]]

    return 2, 'PayloadType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PayloadType

    do -- C Definitions
        ffi.cdef [[
            cstr        PayloadType_ToString(PayloadType);
        ]]
    end

    do -- Global Symbol Table
        PayloadType = {
            Lua         = 0,
            Bool        = 1,
            I8          = 2,
            U8          = 3,
            I16         = 4,
            U16         = 5,
            I32         = 6,
            U32         = 7,
            I64         = 8,
            U64         = 9,
            F32         = 10,
            F64         = 11,
            String      = 12,
            BoolArray   = 13,
            I8Array     = 14,
            U8Array     = 15,
            I16Array    = 16,
            U16Array    = 17,
            I32Array    = 18,
            U32Array    = 19,
            I64Array    = 20,
            U64Array    = 21,
            F32Array    = 22,
            F64Array    = 23,
            StringArray = 24,
            Table       = 25,

            ToString    = libphx.PayloadType_ToString,
        }

        if onDef_PayloadType then onDef_PayloadType(PayloadType, mt) end
        PayloadType = setmetatable(PayloadType, mt)
    end

    return PayloadType
end

return Loader
