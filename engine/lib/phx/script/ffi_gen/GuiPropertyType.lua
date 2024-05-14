-- GuiPropertyType -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 GuiPropertyType;
    ]]

    return 2, 'GuiPropertyType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GuiPropertyType

    do -- C Definitions
        ffi.cdef [[
            GuiPropertyType GuiPropertyType_Bool;
            GuiPropertyType GuiPropertyType_I8;
            GuiPropertyType GuiPropertyType_U8;
            GuiPropertyType GuiPropertyType_I16;
            GuiPropertyType GuiPropertyType_U16;
            GuiPropertyType GuiPropertyType_I32;
            GuiPropertyType GuiPropertyType_U32;
            GuiPropertyType GuiPropertyType_I64;
            GuiPropertyType GuiPropertyType_U64;
            GuiPropertyType GuiPropertyType_F32;
            GuiPropertyType GuiPropertyType_F64;
            GuiPropertyType GuiPropertyType_Vec2;
            GuiPropertyType GuiPropertyType_Vec3;
            GuiPropertyType GuiPropertyType_Vec4;
            GuiPropertyType GuiPropertyType_IVec2;
            GuiPropertyType GuiPropertyType_IVec3;
            GuiPropertyType GuiPropertyType_IVec4;
            GuiPropertyType GuiPropertyType_UVec2;
            GuiPropertyType GuiPropertyType_UVec3;
            GuiPropertyType GuiPropertyType_UVec4;
            GuiPropertyType GuiPropertyType_DVec2;
            GuiPropertyType GuiPropertyType_DVec3;
            GuiPropertyType GuiPropertyType_DVec4;
            GuiPropertyType GuiPropertyType_Color;
            GuiPropertyType GuiPropertyType_Box3;
            GuiPropertyType GuiPropertyType_String;
            GuiPropertyType GuiPropertyType_Font;

            cstr            GuiPropertyType_ToString(GuiPropertyType);
        ]]
    end

    do -- Global Symbol Table
        GuiPropertyType = {
            Bool     = libphx.GuiPropertyType_Bool,
            I8       = libphx.GuiPropertyType_I8,
            U8       = libphx.GuiPropertyType_U8,
            I16      = libphx.GuiPropertyType_I16,
            U16      = libphx.GuiPropertyType_U16,
            I32      = libphx.GuiPropertyType_I32,
            U32      = libphx.GuiPropertyType_U32,
            I64      = libphx.GuiPropertyType_I64,
            U64      = libphx.GuiPropertyType_U64,
            F32      = libphx.GuiPropertyType_F32,
            F64      = libphx.GuiPropertyType_F64,
            Vec2     = libphx.GuiPropertyType_Vec2,
            Vec3     = libphx.GuiPropertyType_Vec3,
            Vec4     = libphx.GuiPropertyType_Vec4,
            IVec2    = libphx.GuiPropertyType_IVec2,
            IVec3    = libphx.GuiPropertyType_IVec3,
            IVec4    = libphx.GuiPropertyType_IVec4,
            UVec2    = libphx.GuiPropertyType_UVec2,
            UVec3    = libphx.GuiPropertyType_UVec3,
            UVec4    = libphx.GuiPropertyType_UVec4,
            DVec2    = libphx.GuiPropertyType_DVec2,
            DVec3    = libphx.GuiPropertyType_DVec3,
            DVec4    = libphx.GuiPropertyType_DVec4,
            Color    = libphx.GuiPropertyType_Color,
            Box3     = libphx.GuiPropertyType_Box3,
            String   = libphx.GuiPropertyType_String,
            Font     = libphx.GuiPropertyType_Font,

            ToString = libphx.GuiPropertyType_ToString,
        }

        if onDef_GuiPropertyType then onDef_GuiPropertyType(GuiPropertyType, mt) end
        GuiPropertyType = setmetatable(GuiPropertyType, mt)
    end

    return GuiPropertyType
end

return Loader
