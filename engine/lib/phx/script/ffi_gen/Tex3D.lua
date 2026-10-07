-- AUTO GENERATED. DO NOT MODIFY!
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
            Tex3D*    Tex3D_Create       (Renderer* r, int sx, int sy, int sz, TexFormat format);
            TexView*  Tex3D_View         (Tex3D const*);
            TexView*  Tex3D_LayerView    (Tex3D const*, int layer);
            TexView*  Tex3D_LayerMipView (Tex3D const*, int layer, int level);
            void      Tex3D_GenMipmap    (Tex3D*, Renderer* r);
            Bytes*    Tex3D_GetDataBytes (Tex3D*, Renderer* r, PixelFormat pf, DataFormat df);
            TexFormat Tex3D_GetFormat    (Tex3D const*);
            Vec3i     Tex3D_GetSize      (Tex3D const*);
            Vec3i     Tex3D_GetSizeLevel (Tex3D const*, int level);
            void      Tex3D_SetDataBytes (Tex3D*, Renderer* r, Bytes* data, PixelFormat pf, DataFormat df);
        ]]
    end

    do -- Global Symbol Table
        Tex3D = {
            Create       = function(r, sx, sy, sz, format)
                local _instance = libphx.Tex3D_Create(r, sx, sy, sz, format)
                return Core.ManagedObject(_instance, libphx.Tex3D_Free)
            end,
        }

        if onDef_Tex3D then onDef_Tex3D(Tex3D, mt) end
        Tex3D = setmetatable(Tex3D, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Tex3D')
        local mt = {
            __index = {
                view         = function(self)
                    local _instance = libphx.Tex3D_View(self)
                    return Core.ManagedObject(_instance, libphx.TexView_Free)
                end,
                layerView    = function(self, layer)
                    local _instance = libphx.Tex3D_LayerView(self, layer)
                    return Core.ManagedObject(_instance, libphx.TexView_Free)
                end,
                layerMipView = function(self, layer, level)
                    local _instance = libphx.Tex3D_LayerMipView(self, layer, level)
                    return Core.ManagedObject(_instance, libphx.TexView_Free)
                end,
                genMipmap    = libphx.Tex3D_GenMipmap,
                getDataBytes = function(self, r, pf, df)
                    local _instance = libphx.Tex3D_GetDataBytes(self, r, pf, df)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                getFormat    = libphx.Tex3D_GetFormat,
                getSize      = libphx.Tex3D_GetSize,
                getSizeLevel = libphx.Tex3D_GetSizeLevel,
                setDataBytes = libphx.Tex3D_SetDataBytes,
            },
        }

        if onDef_Tex3D_t then onDef_Tex3D_t(t, mt) end
        Tex3D_t = ffi.metatype(t, mt)
    end

    return Tex3D
end

return Loader
