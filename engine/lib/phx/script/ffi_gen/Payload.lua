-- Payload ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Payload {} Payload;
    ]]

    return 1, 'Payload'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Payload

    do -- C Definitions
        ffi.cdef [[
            void                Payload_Free            (Payload*);
            Payload*            Payload_FromLua         (uint64 value);
            uint64              Payload_GetLua          (Payload const*);
            Payload*            Payload_FromBool        (bool value);
            bool                Payload_GetBool         (Payload const*);
            Payload*            Payload_FromI8          (int8 value);
            int8                Payload_GetI8           (Payload const*);
            Payload*            Payload_FromU8          (uint8 value);
            uint8               Payload_GetU8           (Payload const*);
            Payload*            Payload_FromI16         (int16 value);
            int16               Payload_GetI16          (Payload const*);
            Payload*            Payload_FromU16         (uint16 value);
            uint16              Payload_GetU16          (Payload const*);
            Payload*            Payload_FromI32         (int value);
            int                 Payload_GetI32          (Payload const*);
            Payload*            Payload_FromU32         (uint32 value);
            uint32              Payload_GetU32          (Payload const*);
            Payload*            Payload_FromI64         (int64 value);
            int64               Payload_GetI64          (Payload const*);
            Payload*            Payload_FromU64         (uint64 value);
            uint64              Payload_GetU64          (Payload const*);
            Payload*            Payload_FromF32         (float value);
            float               Payload_GetF32          (Payload const*);
            Payload*            Payload_FromF64         (double value);
            double              Payload_GetF64          (Payload const*);
            Payload*            Payload_FromString      (cstr value);
            cstr                Payload_GetString       (Payload const*);
            Payload*            Payload_FromBoolArray   (bool const* value, uint64 value_size);
            void                Payload_ForEachBool     (Payload const*, void (*)(bool));
            Payload*            Payload_FromI8Array     (int8 const* value, uint64 value_size);
            void                Payload_ForEachI8       (Payload const*, void (*)(int8));
            Payload*            Payload_FromU8Array     (uint8 const* value, uint64 value_size);
            void                Payload_ForEachU8       (Payload const*, void (*)(uint8));
            Payload*            Payload_FromI16Array    (int16 const* value, uint64 value_size);
            void                Payload_ForEachI16      (Payload const*, void (*)(int16));
            Payload*            Payload_FromU16Array    (uint16 const* value, uint64 value_size);
            void                Payload_ForEachU16      (Payload const*, void (*)(uint16));
            Payload*            Payload_FromI32Array    (int const* value, uint64 value_size);
            void                Payload_ForEachI32      (Payload const*, void (*)(int));
            Payload*            Payload_FromU32Array    (uint32 const* value, uint64 value_size);
            void                Payload_ForEachU32      (Payload const*, void (*)(uint32));
            Payload*            Payload_FromI64Array    (int64 const* value, uint64 value_size);
            void                Payload_ForEachI64      (Payload const*, void (*)(int64));
            Payload*            Payload_FromU64Array    (uint64 const* value, uint64 value_size);
            void                Payload_ForEachU64      (Payload const*, void (*)(uint64));
            Payload*            Payload_FromF32Array    (float const* value, uint64 value_size);
            void                Payload_ForEachF32      (Payload const*, void (*)(float));
            Payload*            Payload_FromF64Array    (double const* value, uint64 value_size);
            void                Payload_ForEachF64      (Payload const*, void (*)(double));
            Payload*            Payload_FromStringArray (cstr* value, uint64 value_size);
            void                Payload_ForEachString   (Payload const*, void (*)(cstr));
            Payload*            Payload_FromTable       (PayloadTable* value);
            PayloadTable const* Payload_GetTable        (Payload const*);
            PayloadType         Payload_GetType         (Payload const*);
        ]]
    end

    do -- Global Symbol Table
        Payload = {
            FromLua         = function(value)
                local _instance = libphx.Payload_FromLua(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromBool        = function(value)
                local _instance = libphx.Payload_FromBool(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI8          = function(value)
                local _instance = libphx.Payload_FromI8(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU8          = function(value)
                local _instance = libphx.Payload_FromU8(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI16         = function(value)
                local _instance = libphx.Payload_FromI16(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU16         = function(value)
                local _instance = libphx.Payload_FromU16(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI32         = function(value)
                local _instance = libphx.Payload_FromI32(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU32         = function(value)
                local _instance = libphx.Payload_FromU32(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI64         = function(value)
                local _instance = libphx.Payload_FromI64(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU64         = function(value)
                local _instance = libphx.Payload_FromU64(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromF32         = function(value)
                local _instance = libphx.Payload_FromF32(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromF64         = function(value)
                local _instance = libphx.Payload_FromF64(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromString      = function(value)
                local _instance = libphx.Payload_FromString(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromBoolArray   = function(value)
                local _instance = libphx.Payload_FromBoolArray(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI8Array     = function(value)
                local _instance = libphx.Payload_FromI8Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU8Array     = function(value)
                local _instance = libphx.Payload_FromU8Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI16Array    = function(value)
                local _instance = libphx.Payload_FromI16Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU16Array    = function(value)
                local _instance = libphx.Payload_FromU16Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI32Array    = function(value)
                local _instance = libphx.Payload_FromI32Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU32Array    = function(value)
                local _instance = libphx.Payload_FromU32Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromI64Array    = function(value)
                local _instance = libphx.Payload_FromI64Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromU64Array    = function(value)
                local _instance = libphx.Payload_FromU64Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromF32Array    = function(value)
                local _instance = libphx.Payload_FromF32Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromF64Array    = function(value)
                local _instance = libphx.Payload_FromF64Array(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromStringArray = function(value)
                local _instance = libphx.Payload_FromStringArray(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
            FromTable       = function(value)
                ffi.gc(value, nil)
                local _instance = libphx.Payload_FromTable(value)
                return Core.ManagedObject(_instance, libphx.Payload_Free)
            end,
        }

        if onDef_Payload then onDef_Payload(Payload, mt) end
        Payload = setmetatable(Payload, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Payload')
        local mt = {
            __index = {
                getLua        = libphx.Payload_GetLua,
                getBool       = libphx.Payload_GetBool,
                getI8         = libphx.Payload_GetI8,
                getU8         = libphx.Payload_GetU8,
                getI16        = libphx.Payload_GetI16,
                getU16        = libphx.Payload_GetU16,
                getI32        = libphx.Payload_GetI32,
                getU32        = libphx.Payload_GetU32,
                getI64        = libphx.Payload_GetI64,
                getU64        = libphx.Payload_GetU64,
                getF32        = libphx.Payload_GetF32,
                getF64        = libphx.Payload_GetF64,
                getString     = libphx.Payload_GetString,
                forEachBool   = libphx.Payload_ForEachBool,
                forEachI8     = libphx.Payload_ForEachI8,
                forEachU8     = libphx.Payload_ForEachU8,
                forEachI16    = libphx.Payload_ForEachI16,
                forEachU16    = libphx.Payload_ForEachU16,
                forEachI32    = libphx.Payload_ForEachI32,
                forEachU32    = libphx.Payload_ForEachU32,
                forEachI64    = libphx.Payload_ForEachI64,
                forEachU64    = libphx.Payload_ForEachU64,
                forEachF32    = libphx.Payload_ForEachF32,
                forEachF64    = libphx.Payload_ForEachF64,
                forEachString = libphx.Payload_ForEachString,
                getTable      = libphx.Payload_GetTable,
                getType       = libphx.Payload_GetType,
            },
        }

        if onDef_Payload_t then onDef_Payload_t(t, mt) end
        Payload_t = ffi.metatype(t, mt)
    end

    return Payload
end

return Loader
