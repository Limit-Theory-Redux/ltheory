-- Tex3D -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Tex3D {} Tex3D;
    ]]

    return 1, 'Tex3D'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Tex3D

    do -- C Definitions
        ffi.cdef [[
            void      Tex3D_Free         (Tex3D*);
            Tex3D*    Tex3D_Create       (int sx, int sy, int sz, TexFormat format);
            void      Tex3D_Pop          (Tex3D const*);
            void      Tex3D_Push         (Tex3D const*, int layer);
            void      Tex3D_PushLevel    (Tex3D const*, int layer, int level);
            void      Tex3D_GenMipmap    (Tex3D*);
            Bytes*    Tex3D_GetDataBytes (Tex3D*, PixelFormat pf, DataFormat df);
            TexFormat Tex3D_GetFormat    (Tex3D const*);
            uint32    Tex3D_GetHandle    (Tex3D const*);
            Vec3i     Tex3D_GetSize      (Tex3D const*);
            Vec3i     Tex3D_GetSizeLevel (Tex3D const*, int level);
            void      Tex3D_SetDataBytes (Tex3D*, Bytes* data, PixelFormat pf, DataFormat df);
            void      Tex3D_SetMagFilter (Tex3D*, TexFilter filter);
            void      Tex3D_SetMinFilter (Tex3D*, TexFilter filter);
            void      Tex3D_SetWrapMode  (Tex3D*, TexWrapMode mode);
        ]]
    end

    do -- Global Symbol Table
        Tex3D = {
            Create       = function(...)
                local instance = libphx.Tex3D_Create(...)
                return Core.ManagedObject(instance, libphx.Tex3D_Free)
            end,
        }

        if onDef_Tex3D then onDef_Tex3D(Tex3D, mt) end
        Tex3D = setmetatable(Tex3D, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Tex3D')
        local mt = {
            __index = {
                pop          = libphx.Tex3D_Pop,
                push         = libphx.Tex3D_Push,
                pushLevel    = libphx.Tex3D_PushLevel,
                genMipmap    = libphx.Tex3D_GenMipmap,
                getDataBytes = function(...)
                    local instance = libphx.Tex3D_GetDataBytes(...)
                    return Core.ManagedObject(instance, libphx.Bytes_Free)
                end,
                getFormat    = libphx.Tex3D_GetFormat,
                getHandle    = libphx.Tex3D_GetHandle,
                getSize      = libphx.Tex3D_GetSize,
                getSizeLevel = libphx.Tex3D_GetSizeLevel,
                setDataBytes = libphx.Tex3D_SetDataBytes,
                setMagFilter = libphx.Tex3D_SetMagFilter,
                setMinFilter = libphx.Tex3D_SetMinFilter,
                setWrapMode  = libphx.Tex3D_SetWrapMode,
            },
        }

        if onDef_Tex3D_t then onDef_Tex3D_t(t, mt) end
        Tex3D_t = ffi.metatype(t, mt)
    end

    return Tex3D
end

return Loader
