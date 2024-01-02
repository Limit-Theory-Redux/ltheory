-- HmGuiPropertyType -----------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 HmGuiPropertyType;
    ]]

    return 2, 'HmGuiPropertyType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local HmGuiPropertyType

    do -- C Definitions
        ffi.cdef [[
            HmGuiPropertyType HmGuiPropertyType_Bool;
            HmGuiPropertyType HmGuiPropertyType_I8;
            HmGuiPropertyType HmGuiPropertyType_U8;
            HmGuiPropertyType HmGuiPropertyType_I16;
            HmGuiPropertyType HmGuiPropertyType_U16;
            HmGuiPropertyType HmGuiPropertyType_I32;
            HmGuiPropertyType HmGuiPropertyType_U32;
            HmGuiPropertyType HmGuiPropertyType_I64;
            HmGuiPropertyType HmGuiPropertyType_U64;
            HmGuiPropertyType HmGuiPropertyType_F32;
            HmGuiPropertyType HmGuiPropertyType_F64;
            HmGuiPropertyType HmGuiPropertyType_Vec2;
            HmGuiPropertyType HmGuiPropertyType_Vec3;
            HmGuiPropertyType HmGuiPropertyType_Vec4;
            HmGuiPropertyType HmGuiPropertyType_IVec2;
            HmGuiPropertyType HmGuiPropertyType_IVec3;
            HmGuiPropertyType HmGuiPropertyType_IVec4;
            HmGuiPropertyType HmGuiPropertyType_UVec2;
            HmGuiPropertyType HmGuiPropertyType_UVec3;
            HmGuiPropertyType HmGuiPropertyType_UVec4;
            HmGuiPropertyType HmGuiPropertyType_DVec2;
            HmGuiPropertyType HmGuiPropertyType_DVec3;
            HmGuiPropertyType HmGuiPropertyType_DVec4;
            HmGuiPropertyType HmGuiPropertyType_Box3;
            HmGuiPropertyType HmGuiPropertyType_String;
            HmGuiPropertyType HmGuiPropertyType_Font;

            cstr              HmGuiPropertyType_ToString(HmGuiPropertyType);
        ]]
    end

    do -- Global Symbol Table
        HmGuiPropertyType = {
            Bool     = libphx.HmGuiPropertyType_Bool,
            I8       = libphx.HmGuiPropertyType_I8,
            U8       = libphx.HmGuiPropertyType_U8,
            I16      = libphx.HmGuiPropertyType_I16,
            U16      = libphx.HmGuiPropertyType_U16,
            I32      = libphx.HmGuiPropertyType_I32,
            U32      = libphx.HmGuiPropertyType_U32,
            I64      = libphx.HmGuiPropertyType_I64,
            U64      = libphx.HmGuiPropertyType_U64,
            F32      = libphx.HmGuiPropertyType_F32,
            F64      = libphx.HmGuiPropertyType_F64,
            Vec2     = libphx.HmGuiPropertyType_Vec2,
            Vec3     = libphx.HmGuiPropertyType_Vec3,
            Vec4     = libphx.HmGuiPropertyType_Vec4,
            IVec2    = libphx.HmGuiPropertyType_IVec2,
            IVec3    = libphx.HmGuiPropertyType_IVec3,
            IVec4    = libphx.HmGuiPropertyType_IVec4,
            UVec2    = libphx.HmGuiPropertyType_UVec2,
            UVec3    = libphx.HmGuiPropertyType_UVec3,
            UVec4    = libphx.HmGuiPropertyType_UVec4,
            DVec2    = libphx.HmGuiPropertyType_DVec2,
            DVec3    = libphx.HmGuiPropertyType_DVec3,
            DVec4    = libphx.HmGuiPropertyType_DVec4,
            Box3     = libphx.HmGuiPropertyType_Box3,
            String   = libphx.HmGuiPropertyType_String,
            Font     = libphx.HmGuiPropertyType_Font,

            ToString = libphx.HmGuiPropertyType_ToString,
        }

        if onDef_HmGuiPropertyType then onDef_HmGuiPropertyType(HmGuiPropertyType, mt) end
        HmGuiPropertyType = setmetatable(HmGuiPropertyType, mt)
    end

    return HmGuiPropertyType
end

return Loader
