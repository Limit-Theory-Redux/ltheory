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
            PayloadType PayloadType_Lua;
            PayloadType PayloadType_Bool;
            PayloadType PayloadType_I8;
            PayloadType PayloadType_U8;
            PayloadType PayloadType_I16;
            PayloadType PayloadType_U16;
            PayloadType PayloadType_I32;
            PayloadType PayloadType_U32;
            PayloadType PayloadType_I64;
            PayloadType PayloadType_U64;
            PayloadType PayloadType_F32;
            PayloadType PayloadType_F64;
            PayloadType PayloadType_String;
            PayloadType PayloadType_BoolArray;
            PayloadType PayloadType_I8Array;
            PayloadType PayloadType_U8Array;
            PayloadType PayloadType_I16Array;
            PayloadType PayloadType_U16Array;
            PayloadType PayloadType_I32Array;
            PayloadType PayloadType_U32Array;
            PayloadType PayloadType_I64Array;
            PayloadType PayloadType_U64Array;
            PayloadType PayloadType_F32Array;
            PayloadType PayloadType_F64Array;
            PayloadType PayloadType_StringArray;
            PayloadType PayloadType_Table;

            cstr        PayloadType_ToString(PayloadType);
        ]]
    end

    do -- Global Symbol Table
        PayloadType = {
            Lua         = libphx.PayloadType_Lua,
            Bool        = libphx.PayloadType_Bool,
            I8          = libphx.PayloadType_I8,
            U8          = libphx.PayloadType_U8,
            I16         = libphx.PayloadType_I16,
            U16         = libphx.PayloadType_U16,
            I32         = libphx.PayloadType_I32,
            U32         = libphx.PayloadType_U32,
            I64         = libphx.PayloadType_I64,
            U64         = libphx.PayloadType_U64,
            F32         = libphx.PayloadType_F32,
            F64         = libphx.PayloadType_F64,
            String      = libphx.PayloadType_String,
            BoolArray   = libphx.PayloadType_BoolArray,
            I8Array     = libphx.PayloadType_I8Array,
            U8Array     = libphx.PayloadType_U8Array,
            I16Array    = libphx.PayloadType_I16Array,
            U16Array    = libphx.PayloadType_U16Array,
            I32Array    = libphx.PayloadType_I32Array,
            U32Array    = libphx.PayloadType_U32Array,
            I64Array    = libphx.PayloadType_I64Array,
            U64Array    = libphx.PayloadType_U64Array,
            F32Array    = libphx.PayloadType_F32Array,
            F64Array    = libphx.PayloadType_F64Array,
            StringArray = libphx.PayloadType_StringArray,
            Table       = libphx.PayloadType_Table,

            ToString    = libphx.PayloadType_ToString,
        }

        if onDef_PayloadType then onDef_PayloadType(PayloadType, mt) end
        PayloadType = setmetatable(PayloadType, mt)
    end

    return PayloadType
end

return Loader
