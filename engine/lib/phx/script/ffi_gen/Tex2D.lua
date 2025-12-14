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
            Tex2D*    Tex2D_Create        (int sx, int sy, TexFormat format);
            Tex2D*    Tex2D_Load          (cstr name);
            Tex2D*    Tex2D_Clone         (Tex2D const*);
            Tex2D*    Tex2D_ScreenCapture ();
            void      Tex2D_Save          (Tex2D*, cstr path);
            void      Tex2D_Pop           (Tex2D const*);
            void      Tex2D_Push          (Tex2D const*);
            void      Tex2D_PushLevel     (Tex2D*, int level);
            void      Tex2D_Clear         (Tex2D*, float r, float g, float b, float a);
            Tex2D*    Tex2D_DeepClone     (Tex2D*);
            void      Tex2D_GenMipmap     (Tex2D*);
            Bytes*    Tex2D_GetDataBytes  (Tex2D const*, PixelFormat pf, DataFormat df);
            TexFormat Tex2D_GetFormat     (Tex2D const*);
            uint32    Tex2D_GetHandle     (Tex2D const*);
            Vec2i     Tex2D_GetSize       (Tex2D const*);
            Vec2i     Tex2D_GetSizeLevel  (Tex2D const*, int level);
            void      Tex2D_SetAnisotropy (Tex2D*, float factor);
            void      Tex2D_SetDataBytes  (Tex2D*, Bytes const* data, PixelFormat pf, DataFormat df);
            void      Tex2D_SetMagFilter  (Tex2D*, TexFilter filter);
            void      Tex2D_SetMinFilter  (Tex2D*, TexFilter filter);
            void      Tex2D_SetMipRange   (Tex2D*, int minLevel, int maxLevel);
            void      Tex2D_SetTexel      (Tex2D*, int x, int y, float r, float g, float b, float a);
            void      Tex2D_SetWrapMode   (Tex2D*, TexWrapMode mode);
            Vec3f     Tex2D_Sample        (Tex2D const*, int x, int y);
        ]]
    end

    do -- Global Symbol Table
        Tex2D = {
            Create        = function(sx, sy, format)
                local _instance = libphx.Tex2D_Create(sx, sy, format)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
            Load          = function(name)
                local _instance = libphx.Tex2D_Load(name)
                return Core.ManagedObject(_instance, libphx.Tex2D_Free)
            end,
            ScreenCapture = function()
                local _instance = libphx.Tex2D_ScreenCapture()
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
                clone         = function(self)
                    local _instance = libphx.Tex2D_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Tex2D_Free)
                end,
                save          = libphx.Tex2D_Save,
                pop           = libphx.Tex2D_Pop,
                push          = libphx.Tex2D_Push,
                pushLevel     = libphx.Tex2D_PushLevel,
                clear         = libphx.Tex2D_Clear,
                deepClone     = function(self)
                    local _instance = libphx.Tex2D_DeepClone(self)
                    return Core.ManagedObject(_instance, libphx.Tex2D_Free)
                end,
                genMipmap     = libphx.Tex2D_GenMipmap,
                getDataBytes  = function(self, pf, df)
                    local _instance = libphx.Tex2D_GetDataBytes(self, pf, df)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                getFormat     = libphx.Tex2D_GetFormat,
                getHandle     = libphx.Tex2D_GetHandle,
                getSize       = libphx.Tex2D_GetSize,
                getSizeLevel  = libphx.Tex2D_GetSizeLevel,
                setAnisotropy = libphx.Tex2D_SetAnisotropy,
                setDataBytes  = libphx.Tex2D_SetDataBytes,
                setMagFilter  = libphx.Tex2D_SetMagFilter,
                setMinFilter  = libphx.Tex2D_SetMinFilter,
                setMipRange   = libphx.Tex2D_SetMipRange,
                setTexel      = libphx.Tex2D_SetTexel,
                setWrapMode   = libphx.Tex2D_SetWrapMode,
                sample        = libphx.Tex2D_Sample,
            },
        }

        if onDef_Tex2D_t then onDef_Tex2D_t(t, mt) end
        Tex2D_t = ffi.metatype(t, mt)
    end

    return Tex2D
end

return Loader
