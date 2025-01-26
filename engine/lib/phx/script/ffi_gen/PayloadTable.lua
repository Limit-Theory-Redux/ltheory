-- AUTO GENERATED. DO NOT MODIFY!
-- PayloadTable ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct PayloadTable {} PayloadTable;
    ]]

    return 1, 'PayloadTable'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PayloadTable

    do -- C Definitions
        ffi.cdef [[
            void           PayloadTable_Free             (PayloadTable*);
            PayloadTable*  PayloadTable_Create           ();
            uint64         PayloadTable_Len              (PayloadTable const*);
            bool           PayloadTable_IsEmpty          (PayloadTable const*);
            bool           PayloadTable_Contains         (PayloadTable const*, cstr name);
            cstr           PayloadTable_GetName          (PayloadTable const*, uint64 index);
            Payload const* PayloadTable_GetPayload       (PayloadTable const*, uint64 index);
            Payload const* PayloadTable_GetPayloadByName (PayloadTable const*, cstr name);
            void           PayloadTable_Add              (PayloadTable*, cstr name, Payload* value);
        ]]
    end

    do -- Global Symbol Table
        PayloadTable = {
            Create           = function()
                local _instance = libphx.PayloadTable_Create()
                return Core.ManagedObject(_instance, libphx.PayloadTable_Free)
            end,
        }

        if onDef_PayloadTable then onDef_PayloadTable(PayloadTable, mt) end
        PayloadTable = setmetatable(PayloadTable, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('PayloadTable')
        local mt = {
            __index = {
                len              = libphx.PayloadTable_Len,
                isEmpty          = libphx.PayloadTable_IsEmpty,
                contains         = libphx.PayloadTable_Contains,
                getName          = libphx.PayloadTable_GetName,
                getPayload       = libphx.PayloadTable_GetPayload,
                getPayloadByName = libphx.PayloadTable_GetPayloadByName,
                add              = function(self, name, value)
                    ffi.gc(value, nil)
                    libphx.PayloadTable_Add(self, name, value)
                end,
            },
        }

        if onDef_PayloadTable_t then onDef_PayloadTable_t(t, mt) end
        PayloadTable_t = ffi.metatype(t, mt)
    end

    return PayloadTable
end

return Loader
