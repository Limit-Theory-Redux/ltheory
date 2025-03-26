-- AUTO GENERATED. DO NOT MODIFY!
-- File ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct File {} File;
    ]]

    return 1, 'File'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local File

    do -- C Definitions
        ffi.cdef [[
            void          File_Free      (File*);
            bool          File_Exists    (cstr path);
            bool          File_IsDir     (cstr path);
            File*         File_Create    (cstr path);
            File*         File_Open      (cstr path);
            bool          File_Close     (File const*);
            Bytes*        File_ReadBytes (cstr path);
            cstr          File_ReadCstr  (cstr path);
            uint64 const* File_Size      (cstr path);
            uint64 const* File_Read      (File*, uint8* data, uint64 data_size);
            uint64 const* File_Write     (File*, uint8 const* data, uint64 data_size);
            uint64 const* File_WriteStr  (File*, cstr data);
            uint8         File_ReadU8    (File*);
            uint16        File_ReadU16   (File*);
            uint32        File_ReadU32   (File*);
            uint64        File_ReadU64   (File*);
            int8          File_ReadI8    (File*);
            int16         File_ReadI16   (File*);
            int           File_ReadI32   (File*);
            int64         File_ReadI64   (File*);
            float         File_ReadF32   (File*);
            double        File_ReadF64   (File*);
            void          File_WriteU8   (File*, uint8 value);
            void          File_WriteU16  (File*, uint16 value);
            void          File_WriteU32  (File*, uint32 value);
            void          File_WriteU64  (File*, uint64 value);
            void          File_WriteI8   (File*, int8 value);
            void          File_WriteI16  (File*, int16 value);
            void          File_WriteI32  (File*, int value);
            void          File_Write64   (File*, int64 value);
            void          File_WriteF32  (File*, float value);
            void          File_WriteF64  (File*, double value);
        ]]
    end

    do -- Global Symbol Table
        File = {
            Exists    = libphx.File_Exists,
            IsDir     = libphx.File_IsDir,
            Create    = function(path)
                local _instance = libphx.File_Create(path)
                return Core.ManagedObject(_instance, libphx.File_Free)
            end,
            Open      = function(path)
                local _instance = libphx.File_Open(path)
                return Core.ManagedObject(_instance, libphx.File_Free)
            end,
            ReadBytes = function(path)
                local _instance = libphx.File_ReadBytes(path)
                return Core.ManagedObject(_instance, libphx.Bytes_Free)
            end,
            ReadCstr  = libphx.File_ReadCstr,
            Size      = libphx.File_Size,
        }

        if onDef_File then onDef_File(File, mt) end
        File = setmetatable(File, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('File')
        local mt = {
            __index = {
                close    = libphx.File_Close,
                read     = libphx.File_Read,
                write    = libphx.File_Write,
                writeStr = libphx.File_WriteStr,
                readU8   = libphx.File_ReadU8,
                readU16  = libphx.File_ReadU16,
                readU32  = libphx.File_ReadU32,
                readU64  = libphx.File_ReadU64,
                readI8   = libphx.File_ReadI8,
                readI16  = libphx.File_ReadI16,
                readI32  = libphx.File_ReadI32,
                readI64  = libphx.File_ReadI64,
                readF32  = libphx.File_ReadF32,
                readF64  = libphx.File_ReadF64,
                writeU8  = libphx.File_WriteU8,
                writeU16 = libphx.File_WriteU16,
                writeU32 = libphx.File_WriteU32,
                writeU64 = libphx.File_WriteU64,
                writeI8  = libphx.File_WriteI8,
                writeI16 = libphx.File_WriteI16,
                writeI32 = libphx.File_WriteI32,
                write64  = libphx.File_Write64,
                writeF32 = libphx.File_WriteF32,
                writeF64 = libphx.File_WriteF64,
            },
        }

        if onDef_File_t then onDef_File_t(t, mt) end
        File_t = ffi.metatype(t, mt)
    end

    return File
end

return Loader
