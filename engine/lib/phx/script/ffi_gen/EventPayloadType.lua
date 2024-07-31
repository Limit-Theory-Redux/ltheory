-- EventPayloadType ------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 EventPayloadType;
    ]]

    return 2, 'EventPayloadType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventPayloadType

    do -- C Definitions
        ffi.cdef [[
            EventPayloadType EventPayloadType_Lua;
            EventPayloadType EventPayloadType_Bool;
            EventPayloadType EventPayloadType_I8;
            EventPayloadType EventPayloadType_U8;
            EventPayloadType EventPayloadType_I16;
            EventPayloadType EventPayloadType_U16;
            EventPayloadType EventPayloadType_I32;
            EventPayloadType EventPayloadType_U32;
            EventPayloadType EventPayloadType_I64;
            EventPayloadType EventPayloadType_U64;
            EventPayloadType EventPayloadType_F32;
            EventPayloadType EventPayloadType_F64;
            EventPayloadType EventPayloadType_String;
            EventPayloadType EventPayloadType_Table;

            cstr             EventPayloadType_ToString(EventPayloadType);
        ]]
    end

    do -- Global Symbol Table
        EventPayloadType = {
            Lua      = libphx.EventPayloadType_Lua,
            Bool     = libphx.EventPayloadType_Bool,
            I8       = libphx.EventPayloadType_I8,
            U8       = libphx.EventPayloadType_U8,
            I16      = libphx.EventPayloadType_I16,
            U16      = libphx.EventPayloadType_U16,
            I32      = libphx.EventPayloadType_I32,
            U32      = libphx.EventPayloadType_U32,
            I64      = libphx.EventPayloadType_I64,
            U64      = libphx.EventPayloadType_U64,
            F32      = libphx.EventPayloadType_F32,
            F64      = libphx.EventPayloadType_F64,
            String   = libphx.EventPayloadType_String,
            Table    = libphx.EventPayloadType_Table,

            ToString = libphx.EventPayloadType_ToString,
        }

        if onDef_EventPayloadType then onDef_EventPayloadType(EventPayloadType, mt) end
        EventPayloadType = setmetatable(EventPayloadType, mt)
    end

    return EventPayloadType
end

return Loader
