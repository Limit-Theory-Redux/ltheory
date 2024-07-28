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
            uint64 const*            EventPayload_AsLua      (EventPayload const*);
            EventPayload*            EventPayload_FromBool   (bool value);
            bool const*              EventPayload_AsBool     (EventPayload const*);
            EventPayload*            EventPayload_FromI8     (int8 value);
            int8 const*              EventPayload_AsI8       (EventPayload const*);
            EventPayload*            EventPayload_FromU8     (uint8 value);
            uint8 const*             EventPayload_AsU8       (EventPayload const*);
            EventPayload*            EventPayload_FromI16    (int16 value);
            int16 const*             EventPayload_AsI16      (EventPayload const*);
            EventPayload*            EventPayload_FromU16    (uint16 value);
            uint16 const*            EventPayload_AsU16      (EventPayload const*);
            EventPayload*            EventPayload_FromI32    (int value);
            int const*               EventPayload_AsI32      (EventPayload const*);
            EventPayload*            EventPayload_FromU32    (uint32 value);
            uint32 const*            EventPayload_AsU32      (EventPayload const*);
            EventPayload*            EventPayload_FromI64    (int64 value);
            int64 const*             EventPayload_AsI64      (EventPayload const*);
            EventPayload*            EventPayload_FromU64    (uint64 value);
            uint64 const*            EventPayload_AsU64      (EventPayload const*);
            EventPayload*            EventPayload_FromF32    (float value);
            float const*             EventPayload_AsF32      (EventPayload const*);
            EventPayload*            EventPayload_FromF64    (double value);
            double const*            EventPayload_AsF64      (EventPayload const*);
            EventPayload*            EventPayload_FromString (cstr value);
            cstr                     EventPayload_AsString   (EventPayload const*);
            EventPayload*            EventPayload_FromTable  (EventPayloadTable* value);
            EventPayloadTable const* EventPayload_AsTable    (EventPayload const*);
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
                asLua    = libphx.EventPayload_AsLua,
                asBool   = libphx.EventPayload_AsBool,
                asI8     = libphx.EventPayload_AsI8,
                asU8     = libphx.EventPayload_AsU8,
                asI16    = libphx.EventPayload_AsI16,
                asU16    = libphx.EventPayload_AsU16,
                asI32    = libphx.EventPayload_AsI32,
                asU32    = libphx.EventPayload_AsU32,
                asI64    = libphx.EventPayload_AsI64,
                asU64    = libphx.EventPayload_AsU64,
                asF32    = libphx.EventPayload_AsF32,
                asF64    = libphx.EventPayload_AsF64,
                asString = libphx.EventPayload_AsString,
                asTable  = libphx.EventPayload_AsTable,
            },
        }

        if onDef_EventPayload_t then onDef_EventPayload_t(t, mt) end
        EventPayload_t = ffi.metatype(t, mt)
    end

    return EventPayload
end

return Loader
