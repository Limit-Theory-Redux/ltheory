-- Bytes -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Bytes {} Bytes;
    ]]

    return 1, 'Bytes'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Bytes

    do -- C Definitions
        ffi.cdef [[
            void   Bytes_Free               (Bytes*);
            Bytes* Bytes_Create             (uint32 size);
            Bytes* Bytes_CreateWithCapacity (uint64 capacity);
            Bytes* Bytes_FromData           (uint8 const* data, uint64 data_size);
            Bytes* Bytes_Load               (cstr path);
            uint32 Bytes_GetSize            (Bytes const*);
            bool   Bytes_IsEmpty            (Bytes const*);
            Bytes* Bytes_Compress           (Bytes const*);
            Bytes* Bytes_Decompress         (Bytes const*);
            void   Bytes_Save               (Bytes const*, cstr path);
            uint32 Bytes_GetCursor          (Bytes const*);
            void   Bytes_Rewind             (Bytes*);
            void   Bytes_SetCursor          (Bytes*, uint32 cursor);
            void   Bytes_Read               (Bytes*, uint8* data, uint64 data_size);
            uint8  Bytes_ReadU8             (Bytes*);
            uint16 Bytes_ReadU16            (Bytes*);
            uint32 Bytes_ReadU32            (Bytes*);
            uint64 Bytes_ReadU64            (Bytes*);
            int8   Bytes_ReadI8             (Bytes*);
            int16  Bytes_ReadI16            (Bytes*);
            int    Bytes_ReadI32            (Bytes*);
            int64  Bytes_ReadI64            (Bytes*);
            float  Bytes_ReadF32            (Bytes*);
            double Bytes_ReadF64            (Bytes*);
            void   Bytes_Write              (Bytes*, uint8 const* data, uint64 data_size);
            void   Bytes_WriteStr           (Bytes*, cstr data);
            void   Bytes_WriteU8            (Bytes*, uint8 value);
            void   Bytes_WriteU16           (Bytes*, uint16 value);
            void   Bytes_WriteU32           (Bytes*, uint32 value);
            void   Bytes_WriteU64           (Bytes*, uint64 value);
            void   Bytes_WriteI8            (Bytes*, int8 value);
            void   Bytes_WriteI16           (Bytes*, int16 value);
            void   Bytes_WriteI32           (Bytes*, int value);
            void   Bytes_WriteI64           (Bytes*, int64 value);
            void   Bytes_WriteF32           (Bytes*, float value);
            void   Bytes_WriteF64           (Bytes*, double value);
        ]]
    end

    do -- Global Symbol Table
        Bytes = {
            Create             = function(size)
                local _instance = libphx.Bytes_Create(size)
                return Core.ManagedObject(_instance, libphx.Bytes_Free)
            end,
            CreateWithCapacity = function(capacity)
                local _instance = libphx.Bytes_CreateWithCapacity(capacity)
                return Core.ManagedObject(_instance, libphx.Bytes_Free)
            end,
            FromData           = function(data)
                local _instance = libphx.Bytes_FromData(data)
                return Core.ManagedObject(_instance, libphx.Bytes_Free)
            end,
            Load               = function(path)
                local _instance = libphx.Bytes_Load(path)
                return Core.ManagedObject(_instance, libphx.Bytes_Free)
            end,
        }

        if onDef_Bytes then onDef_Bytes(Bytes, mt) end
        Bytes = setmetatable(Bytes, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Bytes')
        local mt = {
            __index = {
                getSize    = libphx.Bytes_GetSize,
                isEmpty    = libphx.Bytes_IsEmpty,
                compress   = function(self)
                    local _instance = libphx.Bytes_Compress(self)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                decompress = function(self)
                    local _instance = libphx.Bytes_Decompress(self)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                save       = libphx.Bytes_Save,
                getCursor  = libphx.Bytes_GetCursor,
                rewind     = libphx.Bytes_Rewind,
                setCursor  = libphx.Bytes_SetCursor,
                read       = libphx.Bytes_Read,
                readU8     = libphx.Bytes_ReadU8,
                readU16    = libphx.Bytes_ReadU16,
                readU32    = libphx.Bytes_ReadU32,
                readU64    = libphx.Bytes_ReadU64,
                readI8     = libphx.Bytes_ReadI8,
                readI16    = libphx.Bytes_ReadI16,
                readI32    = libphx.Bytes_ReadI32,
                readI64    = libphx.Bytes_ReadI64,
                readF32    = libphx.Bytes_ReadF32,
                readF64    = libphx.Bytes_ReadF64,
                write      = libphx.Bytes_Write,
                writeStr   = libphx.Bytes_WriteStr,
                writeU8    = libphx.Bytes_WriteU8,
                writeU16   = libphx.Bytes_WriteU16,
                writeU32   = libphx.Bytes_WriteU32,
                writeU64   = libphx.Bytes_WriteU64,
                writeI8    = libphx.Bytes_WriteI8,
                writeI16   = libphx.Bytes_WriteI16,
                writeI32   = libphx.Bytes_WriteI32,
                writeI64   = libphx.Bytes_WriteI64,
                writeF32   = libphx.Bytes_WriteF32,
                writeF64   = libphx.Bytes_WriteF64,
            },
        }

        if onDef_Bytes_t then onDef_Bytes_t(t, mt) end
        Bytes_t = ffi.metatype(t, mt)
    end

    return Bytes
end

return Loader
