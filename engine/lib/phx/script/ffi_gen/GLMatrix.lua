-- GLMatrix --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'GLMatrix'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GLMatrix

    do -- C Definitions
        ffi.cdef [[
            void          GLMatrix_Clear       ();
            void          GLMatrix_Load        (Matrix const* matrix);
            void          GLMatrix_LookAt      (Vec3d const* eye, Vec3d const* at, Vec3d const* up);
            void          GLMatrix_ModeP       ();
            void          GLMatrix_ModeWV      ();
            void          GLMatrix_Mult        (Matrix* matrix);
            void          GLMatrix_Perspective (double fovy, double aspect, double z0, double z1);
            void          GLMatrix_Pop         ();
            void          GLMatrix_Push        ();
            void          GLMatrix_PushClear   ();
            Matrix const* GLMatrix_Get         ();
            void          GLMatrix_RotateX     (double angle);
            void          GLMatrix_RotateY     (double angle);
            void          GLMatrix_RotateZ     (double angle);
            void          GLMatrix_Scale       (double x, double y, double z);
            void          GLMatrix_Translate   (double x, double y, double z);
        ]]
    end

    do -- Global Symbol Table
        GLMatrix = {
            Clear       = libphx.GLMatrix_Clear,
            Load        = libphx.GLMatrix_Load,
            LookAt      = libphx.GLMatrix_LookAt,
            ModeP       = libphx.GLMatrix_ModeP,
            ModeWV      = libphx.GLMatrix_ModeWV,
            Mult        = libphx.GLMatrix_Mult,
            Perspective = libphx.GLMatrix_Perspective,
            Pop         = libphx.GLMatrix_Pop,
            Push        = libphx.GLMatrix_Push,
            PushClear   = libphx.GLMatrix_PushClear,
            Get         = libphx.GLMatrix_Get,
            RotateX     = libphx.GLMatrix_RotateX,
            RotateY     = libphx.GLMatrix_RotateY,
            RotateZ     = libphx.GLMatrix_RotateZ,
            Scale       = libphx.GLMatrix_Scale,
            Translate   = libphx.GLMatrix_Translate,
        }

        if onDef_GLMatrix then onDef_GLMatrix(GLMatrix, mt) end
        GLMatrix = setmetatable(GLMatrix, mt)
    end

    return GLMatrix
end

return Loader
