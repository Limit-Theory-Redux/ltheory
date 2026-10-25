-- AUTO GENERATED. DO NOT MODIFY!
-- Tex2D -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Tex2D {} Tex2D;
    ]]

    return 1, 'Tex2D'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Tex2D

    do -- C Definitions
        ffi.cdef [[
            void      Tex2D_Free          (Tex2D*);
            Tex2D*    Tex2D_Create        (Renderer* r, int sx, int sy, TexFormat format);
            Tex2D*    Tex2D_CreateDesc    (Renderer* r, int sx, int sy, TexFormat format, int mips, uint32 usage);
            Tex2D*    Tex2D_Load          (Renderer* r, cstr name);
            Tex2D*    Tex2D_Clone         (Tex2D const*);
            Tex2D*    Tex2D_ScreenCapture (Renderer* r);
            void      Tex2D_Save          (Tex2D*, Renderer* r, cstr path);
            TexView*  Tex2D_View          (Tex2D const*);
            TexView*  Tex2D_MipView       (Tex2D const*, int level);
            void      Tex2D_Clear         (Tex2D*, Renderer* r, float red, float green, float blue, float alpha);
            Tex2D*    Tex2D_DeepClone     (Tex2D*, Renderer* r);
            void      Tex2D_GenMipmap     (Tex2D*, Renderer* r);
            Bytes*    Tex2D_GetDataBytes  (Tex2D const*, Renderer* r, PixelFormat pf, DataFormat df);
            TexFormat Tex2D_GetFormat     (Tex2D const*);
            Vec2i     Tex2D_GetSize       (Tex2D const*);
            Vec2i     Tex2D_GetSizeLevel  (Tex2D const*, int level);
            void      Tex2D_SetDataBytes  (Tex2D*, Renderer* r, Bytes const* data, PixelFormat pf, DataFormat df);
            void      Tex2D_SetTexel      (Tex2D*, Renderer* r, int x, int y, float red, float green, float blue, float alpha);
        ]]
    end

    do -- Global Symbol Table
        Tex2D = {
            Create        = function(r, sx, sy, format)
                local _instance = libphx.Tex2D_Create(r, sx, sy, format)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
            CreateDesc    = function(r, sx, sy, format, mips, usage)
                local _instance = libphx.Tex2D_CreateDesc(r, sx, sy, format, mips, usage)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
            Load          = function(r, name)
                local _instance = libphx.Tex2D_Load(r, name)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
            ScreenCapture = function(r)
                local _instance = libphx.Tex2D_ScreenCapture(r)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
        }

        if onDef_Tex2D then onDef_Tex2D(Tex2D, mt) end
        Tex2D = setmetatable(Tex2D, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Tex2D')
        local mt = {
            __index = {
                clone        = function(self)
                    local _instance = libphx.Tex2D_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Tex2D_Free)
                end,
                save         = libphx.Tex2D_Save,
                view         = function(self)
                    local _instance = libphx.Tex2D_View(self)
                    return Core.ManagedObject(_instance, libphx.TexView_Free)
                end,
                mipView      = function(self, level)
                    local _instance = libphx.Tex2D_MipView(self, level)
                    return Core.ManagedObject(_instance, libphx.TexView_Free)
                end,
                clear        = libphx.Tex2D_Clear,
                deepClone    = function(self, r)
                    local _instance = libphx.Tex2D_DeepClone(self, r)
                    return Core.ManagedObject(_instance, libphx.Tex2D_Free)
                end,
                genMipmap    = libphx.Tex2D_GenMipmap,
                getDataBytes = function(self, r, pf, df)
                    local _instance = libphx.Tex2D_GetDataBytes(self, r, pf, df)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                getFormat    = libphx.Tex2D_GetFormat,
                getSize      = libphx.Tex2D_GetSize,
                getSizeLevel = libphx.Tex2D_GetSizeLevel,
                setDataBytes = libphx.Tex2D_SetDataBytes,
                setTexel     = libphx.Tex2D_SetTexel,
            },
        }

        if onDef_Tex2D_t then onDef_Tex2D_t(t, mt) end
        Tex2D_t = ffi.metatype(t, mt)
    end

    return Tex2D
end

return Loader
