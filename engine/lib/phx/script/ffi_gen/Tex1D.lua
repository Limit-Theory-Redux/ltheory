-- AUTO GENERATED. DO NOT MODIFY!
-- Tex1D -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Tex1D {} Tex1D;
    ]]

    return 1, 'Tex1D'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Tex1D

    do -- C Definitions
        ffi.cdef [[
            void      Tex1D_Free         (Tex1D*);
            Tex1D*    Tex1D_Create       (int size, TexFormat format);
            Tex1D*    Tex1D_Clone        (Tex1D const*);
            void      Tex1D_GenMipmap    (Tex1D*);
            TexFormat Tex1D_GetFormat    (Tex1D*);
            Bytes*    Tex1D_GetDataBytes (Tex1D*, PixelFormat pf, DataFormat df);
            uint32    Tex1D_GetHandle    (Tex1D const*);
            uint32    Tex1D_GetSize      (Tex1D const*);
            void      Tex1D_SetDataBytes (Tex1D*, Bytes const* data, PixelFormat pf, DataFormat df);
            void      Tex1D_SetMagFilter (Tex1D*, TexFilter filter);
            void      Tex1D_SetMinFilter (Tex1D*, TexFilter filter);
            void      Tex1D_SetTexel     (Tex1D*, int x, float r, float g, float b, float a);
            void      Tex1D_SetWrapMode  (Tex1D*, TexWrapMode mode);
        ]]
    end

    do -- Global Symbol Table
        Tex1D = {
            Create       = function(size, format)
                local _instance = libphx.Tex1D_Create(size, format)
                return Core.ManagedObject(_instance, libphx.Tex1D_Free)
            end,
        }

        if onDef_Tex1D then onDef_Tex1D(Tex1D, mt) end
        Tex1D = setmetatable(Tex1D, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Tex1D')
        local mt = {
            __index = {
                clone        = function(self)
                    local _instance = libphx.Tex1D_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Tex1D_Free)
                end,
                genMipmap    = libphx.Tex1D_GenMipmap,
                getFormat    = libphx.Tex1D_GetFormat,
                getDataBytes = function(self, pf, df)
                    local _instance = libphx.Tex1D_GetDataBytes(self, pf, df)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                getHandle    = libphx.Tex1D_GetHandle,
                getSize      = libphx.Tex1D_GetSize,
                setDataBytes = libphx.Tex1D_SetDataBytes,
                setMagFilter = libphx.Tex1D_SetMagFilter,
                setMinFilter = libphx.Tex1D_SetMinFilter,
                setTexel     = libphx.Tex1D_SetTexel,
                setWrapMode  = libphx.Tex1D_SetWrapMode,
            },
        }

        if onDef_Tex1D_t then onDef_Tex1D_t(t, mt) end
        Tex1D_t = ffi.metatype(t, mt)
    end

    return Tex1D
end

return Loader
