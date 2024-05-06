-- GuiPropertyValue ------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct GuiPropertyValue {} GuiPropertyValue;
    ]]

    return 1, 'GuiPropertyValue'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GuiPropertyValue

    do -- C Definitions
        ffi.cdef [[
            void              GuiPropertyValue_Free       (GuiPropertyValue*);
            GuiPropertyType   GuiPropertyValue_GetType    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromBool   (bool value);
            bool              GuiPropertyValue_GetBool    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromI8     (int8 value);
            int8              GuiPropertyValue_GetI8      (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromU8     (uint8 value);
            uint8             GuiPropertyValue_GetU8      (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromI16    (int16 value);
            int16             GuiPropertyValue_GetI16     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromU16    (uint16 value);
            uint16            GuiPropertyValue_GetU16     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromI32    (int value);
            int               GuiPropertyValue_GetI32     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromU32    (uint32 value);
            uint32            GuiPropertyValue_GetU32     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromI64    (int64 value);
            int64             GuiPropertyValue_GetI64     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromU64    (uint64 value);
            uint64            GuiPropertyValue_GetU64     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromF32    (float value);
            float             GuiPropertyValue_GetF32     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromF64    (double value);
            double            GuiPropertyValue_GetF64     (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromVec2   (Vec2f value);
            Vec2f             GuiPropertyValue_GetVec2    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromVec3   (Vec3f value);
            Vec3f             GuiPropertyValue_GetVec3    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromVec4   (Vec4f value);
            Vec4f             GuiPropertyValue_GetVec4    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromIvec2  (Vec2i value);
            Vec2i             GuiPropertyValue_GetIvec2   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromIvec3  (Vec3i value);
            Vec3i             GuiPropertyValue_GetIvec3   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromIvec4  (Vec4i value);
            Vec4i             GuiPropertyValue_GetIvec4   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromUvec2  (Vec2u value);
            Vec2u             GuiPropertyValue_GetUvec2   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromUvec3  (Vec3u value);
            Vec3u             GuiPropertyValue_GetUvec3   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromUvec4  (Vec4u value);
            Vec4u             GuiPropertyValue_GetUvec4   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromDvec2  (Vec2d value);
            Vec2d             GuiPropertyValue_GetDvec2   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromDvec3  (Vec3d value);
            Vec3d             GuiPropertyValue_GetDvec3   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromDvec4  (Vec4d value);
            Vec4d             GuiPropertyValue_GetDvec4   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromColor  (Color* value);
            Color*            GuiPropertyValue_GetColor   (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromBox3   (Box3f value);
            Box3f             GuiPropertyValue_GetBox3    (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromString (cstr value);
            cstr              GuiPropertyValue_GetString  (GuiPropertyValue const*);
            GuiPropertyValue* GuiPropertyValue_FromFont   (Font* value);
            Font*             GuiPropertyValue_GetFont    (GuiPropertyValue const*);
        ]]
    end

    do -- Global Symbol Table
        GuiPropertyValue = {
            FromBool   = function(...)
                local instance = libphx.GuiPropertyValue_FromBool(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromI8     = function(...)
                local instance = libphx.GuiPropertyValue_FromI8(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromU8     = function(...)
                local instance = libphx.GuiPropertyValue_FromU8(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromI16    = function(...)
                local instance = libphx.GuiPropertyValue_FromI16(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromU16    = function(...)
                local instance = libphx.GuiPropertyValue_FromU16(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromI32    = function(...)
                local instance = libphx.GuiPropertyValue_FromI32(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromU32    = function(...)
                local instance = libphx.GuiPropertyValue_FromU32(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromI64    = function(...)
                local instance = libphx.GuiPropertyValue_FromI64(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromU64    = function(...)
                local instance = libphx.GuiPropertyValue_FromU64(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromF32    = function(...)
                local instance = libphx.GuiPropertyValue_FromF32(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromF64    = function(...)
                local instance = libphx.GuiPropertyValue_FromF64(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromVec2   = function(...)
                local instance = libphx.GuiPropertyValue_FromVec2(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromVec3   = function(...)
                local instance = libphx.GuiPropertyValue_FromVec3(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromVec4   = function(...)
                local instance = libphx.GuiPropertyValue_FromVec4(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromIvec2  = function(...)
                local instance = libphx.GuiPropertyValue_FromIvec2(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromIvec3  = function(...)
                local instance = libphx.GuiPropertyValue_FromIvec3(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromIvec4  = function(...)
                local instance = libphx.GuiPropertyValue_FromIvec4(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromUvec2  = function(...)
                local instance = libphx.GuiPropertyValue_FromUvec2(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromUvec3  = function(...)
                local instance = libphx.GuiPropertyValue_FromUvec3(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromUvec4  = function(...)
                local instance = libphx.GuiPropertyValue_FromUvec4(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromDvec2  = function(...)
                local instance = libphx.GuiPropertyValue_FromDvec2(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromDvec3  = function(...)
                local instance = libphx.GuiPropertyValue_FromDvec3(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromDvec4  = function(...)
                local instance = libphx.GuiPropertyValue_FromDvec4(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromColor  = function(...)
                local instance = libphx.GuiPropertyValue_FromColor(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromBox3   = function(...)
                local instance = libphx.GuiPropertyValue_FromBox3(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromString = function(...)
                local instance = libphx.GuiPropertyValue_FromString(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
            FromFont   = function(...)
                local instance = libphx.GuiPropertyValue_FromFont(...)
                return Core.ManagedObject(instance, libphx.GuiPropertyValue_Free)
            end,
        }

        if onDef_GuiPropertyValue then onDef_GuiPropertyValue(GuiPropertyValue, mt) end
        GuiPropertyValue = setmetatable(GuiPropertyValue, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('GuiPropertyValue')
        local mt = {
            __index = {
                getType   = libphx.GuiPropertyValue_GetType,
                getBool   = libphx.GuiPropertyValue_GetBool,
                getI8     = libphx.GuiPropertyValue_GetI8,
                getU8     = libphx.GuiPropertyValue_GetU8,
                getI16    = libphx.GuiPropertyValue_GetI16,
                getU16    = libphx.GuiPropertyValue_GetU16,
                getI32    = libphx.GuiPropertyValue_GetI32,
                getU32    = libphx.GuiPropertyValue_GetU32,
                getI64    = libphx.GuiPropertyValue_GetI64,
                getU64    = libphx.GuiPropertyValue_GetU64,
                getF32    = libphx.GuiPropertyValue_GetF32,
                getF64    = libphx.GuiPropertyValue_GetF64,
                getVec2   = libphx.GuiPropertyValue_GetVec2,
                getVec3   = libphx.GuiPropertyValue_GetVec3,
                getVec4   = libphx.GuiPropertyValue_GetVec4,
                getIvec2  = libphx.GuiPropertyValue_GetIvec2,
                getIvec3  = libphx.GuiPropertyValue_GetIvec3,
                getIvec4  = libphx.GuiPropertyValue_GetIvec4,
                getUvec2  = libphx.GuiPropertyValue_GetUvec2,
                getUvec3  = libphx.GuiPropertyValue_GetUvec3,
                getUvec4  = libphx.GuiPropertyValue_GetUvec4,
                getDvec2  = libphx.GuiPropertyValue_GetDvec2,
                getDvec3  = libphx.GuiPropertyValue_GetDvec3,
                getDvec4  = libphx.GuiPropertyValue_GetDvec4,
                getColor  = function(...)
                    local instance = libphx.GuiPropertyValue_GetColor(...)
                    return Core.ManagedObject(instance, libphx.Color_Free)
                end,
                getBox3   = libphx.GuiPropertyValue_GetBox3,
                getString = libphx.GuiPropertyValue_GetString,
                getFont   = function(...)
                    local instance = libphx.GuiPropertyValue_GetFont(...)
                    return Core.ManagedObject(instance, libphx.Font_Free)
                end,
            },
        }

        if onDef_GuiPropertyValue_t then onDef_GuiPropertyValue_t(t, mt) end
        GuiPropertyValue_t = ffi.metatype(t, mt)
    end

    return GuiPropertyValue
end

return Loader
