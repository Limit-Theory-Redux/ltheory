-- EventPayloadTable -----------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EventPayloadTable {} EventPayloadTable;
    ]]

    return 1, 'EventPayloadTable'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventPayloadTable

    do -- C Definitions
        ffi.cdef [[
            void               EventPayloadTable_Free   (EventPayloadTable*);
            EventPayloadTable* EventPayloadTable_Create ();
            void               EventPayloadTable_Add    (EventPayloadTable*, cstr name, EventPayload* value);
        ]]
    end

    do -- Global Symbol Table
        EventPayloadTable = {
            Create = function(...)
                local instance = libphx.EventPayloadTable_Create(...)
                return Core.ManagedObject(instance, libphx.EventPayloadTable_Free)
            end,
        }

        if onDef_EventPayloadTable then onDef_EventPayloadTable(EventPayloadTable, mt) end
        EventPayloadTable = setmetatable(EventPayloadTable, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EventPayloadTable')
        local mt = {
            __index = {
                add  = libphx.EventPayloadTable_Add,
            },
        }

        if onDef_EventPayloadTable_t then onDef_EventPayloadTable_t(t, mt) end
        EventPayloadTable_t = ffi.metatype(t, mt)
    end

    return EventPayloadTable
end

return Loader
