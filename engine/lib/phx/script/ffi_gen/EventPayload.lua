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
            void                     EventPayload_Free          (EventPayload*);
            EventPayload*            EventPayload_FromLua       (uint64 value);
            uint64                   EventPayload_GetLua        (EventPayload const*);
            EventPayload*            EventPayload_FromBool      (bool value);
            bool                     EventPayload_GetBool       (EventPayload const*);
            EventPayload*            EventPayload_FromI8        (int8 value);
            int8                     EventPayload_GetI8         (EventPayload const*);
            EventPayload*            EventPayload_FromU8        (uint8 value);
            uint8                    EventPayload_GetU8         (EventPayload const*);
            EventPayload*            EventPayload_FromI16       (int16 value);
            int16                    EventPayload_GetI16        (EventPayload const*);
            EventPayload*            EventPayload_FromU16       (uint16 value);
            uint16                   EventPayload_GetU16        (EventPayload const*);
            EventPayload*            EventPayload_FromI32       (int value);
            int                      EventPayload_GetI32        (EventPayload const*);
            EventPayload*            EventPayload_FromU32       (uint32 value);
            uint32                   EventPayload_GetU32        (EventPayload const*);
            EventPayload*            EventPayload_FromI64       (int64 value);
            int64                    EventPayload_GetI64        (EventPayload const*);
            EventPayload*            EventPayload_FromU64       (uint64 value);
            uint64                   EventPayload_GetU64        (EventPayload const*);
            EventPayload*            EventPayload_FromF32       (float value);
            float                    EventPayload_GetF32        (EventPayload const*);
            EventPayload*            EventPayload_FromF64       (double value);
            double                   EventPayload_GetF64        (EventPayload const*);
            EventPayload*            EventPayload_FromString    (cstr value);
            cstr                     EventPayload_GetString     (EventPayload const*);
            EventPayload*            EventPayload_FromBoolArray (bool const* value, uint64 value_size);
            void                     EventPayload_ForEachBool   (EventPayload const*, void (*)(bool));
            EventPayload*            EventPayload_FromI8Array   (int8 const* value, uint64 value_size);
            void                     EventPayload_ForEachI8     (EventPayload const*, void (*)(int8));
            EventPayload*            EventPayload_FromU8Array   (uint8 const* value, uint64 value_size);
            void                     EventPayload_ForEachU8     (EventPayload const*, void (*)(uint8));
            EventPayload*            EventPayload_FromI16Array  (int16 const* value, uint64 value_size);
            void                     EventPayload_ForEachI16    (EventPayload const*, void (*)(int16));
            EventPayload*            EventPayload_FromU16Array  (uint16 const* value, uint64 value_size);
            void                     EventPayload_ForEachU16    (EventPayload const*, void (*)(uint16));
            EventPayload*            EventPayload_FromI32Array  (int const* value, uint64 value_size);
            void                     EventPayload_ForEachI32    (EventPayload const*, void (*)(int));
            EventPayload*            EventPayload_FromU32Array  (uint32 const* value, uint64 value_size);
            void                     EventPayload_ForEachU32    (EventPayload const*, void (*)(uint32));
            EventPayload*            EventPayload_FromI64Array  (int64 const* value, uint64 value_size);
            void                     EventPayload_ForEachI64    (EventPayload const*, void (*)(int64));
            EventPayload*            EventPayload_FromU64Array  (uint64 const* value, uint64 value_size);
            void                     EventPayload_ForEachU64    (EventPayload const*, void (*)(uint64));
            EventPayload*            EventPayload_FromF32Array  (float const* value, uint64 value_size);
            void                     EventPayload_ForEachF32    (EventPayload const*, void (*)(float));
            EventPayload*            EventPayload_FromF64Array  (double const* value, uint64 value_size);
            void                     EventPayload_ForEachF64    (EventPayload const*, void (*)(double));
            void                     EventPayload_ForEachString (EventPayload const*, void (*)(cstr));
            EventPayload*            EventPayload_FromTable     (EventPayloadTable* value);
            EventPayloadTable const* EventPayload_GetTable      (EventPayload const*);
            EventPayloadType         EventPayload_GetType       (EventPayload const*);
        ]]
    end

    do -- Global Symbol Table
        EventPayload = {
            FromLua       = function(...)
                local instance = libphx.EventPayload_FromLua(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromBool      = function(...)
                local instance = libphx.EventPayload_FromBool(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI8        = function(...)
                local instance = libphx.EventPayload_FromI8(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU8        = function(...)
                local instance = libphx.EventPayload_FromU8(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI16       = function(...)
                local instance = libphx.EventPayload_FromI16(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU16       = function(...)
                local instance = libphx.EventPayload_FromU16(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI32       = function(...)
                local instance = libphx.EventPayload_FromI32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU32       = function(...)
                local instance = libphx.EventPayload_FromU32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI64       = function(...)
                local instance = libphx.EventPayload_FromI64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU64       = function(...)
                local instance = libphx.EventPayload_FromU64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF32       = function(...)
                local instance = libphx.EventPayload_FromF32(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF64       = function(...)
                local instance = libphx.EventPayload_FromF64(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromString    = function(...)
                local instance = libphx.EventPayload_FromString(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromBoolArray = function(...)
                local instance = libphx.EventPayload_FromBoolArray(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI8Array   = function(...)
                local instance = libphx.EventPayload_FromI8Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU8Array   = function(...)
                local instance = libphx.EventPayload_FromU8Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI16Array  = function(...)
                local instance = libphx.EventPayload_FromI16Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU16Array  = function(...)
                local instance = libphx.EventPayload_FromU16Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI32Array  = function(...)
                local instance = libphx.EventPayload_FromI32Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU32Array  = function(...)
                local instance = libphx.EventPayload_FromU32Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromI64Array  = function(...)
                local instance = libphx.EventPayload_FromI64Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromU64Array  = function(...)
                local instance = libphx.EventPayload_FromU64Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF32Array  = function(...)
                local instance = libphx.EventPayload_FromF32Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromF64Array  = function(...)
                local instance = libphx.EventPayload_FromF64Array(...)
                return Core.ManagedObject(instance, libphx.EventPayload_Free)
            end,
            FromTable     = function(...)
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
                getLua        = libphx.EventPayload_GetLua,
                getBool       = libphx.EventPayload_GetBool,
                getI8         = libphx.EventPayload_GetI8,
                getU8         = libphx.EventPayload_GetU8,
                getI16        = libphx.EventPayload_GetI16,
                getU16        = libphx.EventPayload_GetU16,
                getI32        = libphx.EventPayload_GetI32,
                getU32        = libphx.EventPayload_GetU32,
                getI64        = libphx.EventPayload_GetI64,
                getU64        = libphx.EventPayload_GetU64,
                getF32        = libphx.EventPayload_GetF32,
                getF64        = libphx.EventPayload_GetF64,
                getString     = libphx.EventPayload_GetString,
                forEachBool   = libphx.EventPayload_ForEachBool,
                forEachI8     = libphx.EventPayload_ForEachI8,
                forEachU8     = libphx.EventPayload_ForEachU8,
                forEachI16    = libphx.EventPayload_ForEachI16,
                forEachU16    = libphx.EventPayload_ForEachU16,
                forEachI32    = libphx.EventPayload_ForEachI32,
                forEachU32    = libphx.EventPayload_ForEachU32,
                forEachI64    = libphx.EventPayload_ForEachI64,
                forEachU64    = libphx.EventPayload_ForEachU64,
                forEachF32    = libphx.EventPayload_ForEachF32,
                forEachF64    = libphx.EventPayload_ForEachF64,
                forEachString = libphx.EventPayload_ForEachString,
                getTable      = libphx.EventPayload_GetTable,
                getType       = libphx.EventPayload_GetType,
            },
        }

        if onDef_EventPayload_t then onDef_EventPayload_t(t, mt) end
        EventPayload_t = ffi.metatype(t, mt)
    end

    return EventPayload
end

return Loader
