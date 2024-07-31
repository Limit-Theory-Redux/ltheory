-- EventPayload ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EventPayload {} EventPayload;
    ]]

    return 1, 'EventPayload'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventPayload

    do -- C Definitions
        ffi.cdef [[
            void                     EventPayload_Free       (EventPayload*);
            EventPayload*            EventPayload_FromLua    (uint64 value);
            uint64                   EventPayload_GetLua     (EventPayload const*);
            EventPayload*            EventPayload_FromBool   (bool value);
            bool                     EventPayload_GetBool    (EventPayload const*);
            EventPayload*            EventPayload_FromI8     (int8 value);
            int8                     EventPayload_GetI8      (EventPayload const*);
            EventPayload*            EventPayload_FromU8     (uint8 value);
            uint8                    EventPayload_GetU8      (EventPayload const*);
            EventPayload*            EventPayload_FromI16    (int16 value);
            int16                    EventPayload_GetI16     (EventPayload const*);
            EventPayload*            EventPayload_FromU16    (uint16 value);
            uint16                   EventPayload_GetU16     (EventPayload const*);
            EventPayload*            EventPayload_FromI32    (int value);
            int                      EventPayload_GetI32     (EventPayload const*);
            EventPayload*            EventPayload_FromU32    (uint32 value);
            uint32                   EventPayload_GetU32     (EventPayload const*);
            EventPayload*            EventPayload_FromI64    (int64 value);
            int64                    EventPayload_GetI64     (EventPayload const*);
            EventPayload*            EventPayload_FromU64    (uint64 value);
            uint64                   EventPayload_GetU64     (EventPayload const*);
            EventPayload*            EventPayload_FromF32    (float value);
            float                    EventPayload_GetF32     (EventPayload const*);
            EventPayload*            EventPayload_FromF64    (double value);
            double                   EventPayload_GetF64     (EventPayload const*);
            EventPayload*            EventPayload_FromString (cstr value);
            cstr                     EventPayload_GetString  (EventPayload const*);
            EventPayload*            EventPayload_FromTable  (EventPayloadTable* value);
            EventPayloadTable const* EventPayload_GetTable   (EventPayload const*);
            EventPayloadType         EventPayload_GetType    (EventPayload const*);
        ]]
    end

    do -- Global Symbol Table
        EventPayload = {
            FromLua    = function(...)
                local instance = libphx.EventPayload_FromLua(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromBool   = function(...)
                local instance = libphx.EventPayload_FromBool(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI8     = function(...)
                local instance = libphx.EventPayload_FromI8(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU8     = function(...)
                local instance = libphx.EventPayload_FromU8(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI16    = function(...)
                local instance = libphx.EventPayload_FromI16(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU16    = function(...)
                local instance = libphx.EventPayload_FromU16(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI32    = function(...)
                local instance = libphx.EventPayload_FromI32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU32    = function(...)
                local instance = libphx.EventPayload_FromU32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI64    = function(...)
                local instance = libphx.EventPayload_FromI64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU64    = function(...)
                local instance = libphx.EventPayload_FromU64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF32    = function(...)
                local instance = libphx.EventPayload_FromF32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF64    = function(...)
                local instance = libphx.EventPayload_FromF64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromString = function(...)
                local instance = libphx.EventPayload_FromString(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromTable  = function(...)
                local instance = libphx.EventPayload_FromTable(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
        }

        if onDef_EventPayload then onDef_EventPayload(EventPayload, mt) end
        EventPayload = setmetatable(EventPayload, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EventPayload')
        local mt = {
            __index = {
                getLua    = libphx.EventPayload_GetLua,
                getBool   = libphx.EventPayload_GetBool,
                getI8     = libphx.EventPayload_GetI8,
                getU8     = libphx.EventPayload_GetU8,
                getI16    = libphx.EventPayload_GetI16,
                getU16    = libphx.EventPayload_GetU16,
                getI32    = libphx.EventPayload_GetI32,
                getU32    = libphx.EventPayload_GetU32,
                getI64    = libphx.EventPayload_GetI64,
                getU64    = libphx.EventPayload_GetU64,
                getF32    = libphx.EventPayload_GetF32,
                getF64    = libphx.EventPayload_GetF64,
                getString = libphx.EventPayload_GetString,
                getTable  = libphx.EventPayload_GetTable,
                getType   = libphx.EventPayload_GetType,
            },
        }

        if onDef_EventPayload_t then onDef_EventPayload_t(t, mt) end
        EventPayload_t = ffi.metatype(t, mt)
    end

    return EventPayload
end

return Loader
