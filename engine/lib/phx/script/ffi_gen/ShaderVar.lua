-- ShaderVar -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'ShaderVar'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ShaderVar

    do -- C Definitions
        ffi.cdef [[
            void ShaderVar_PushFloat   (cstr name, float x);
            void ShaderVar_PushFloat2  (cstr name, float x, float y);
            void ShaderVar_PushFloat3  (cstr name, float x, float y, float z);
            void ShaderVar_PushFloat4  (cstr name, float x, float y, float z, float w);
            void ShaderVar_PushInt     (cstr name, int x);
            void ShaderVar_PushInt2    (cstr name, int x, int y);
            void ShaderVar_PushInt3    (cstr name, int x, int y, int z);
            void ShaderVar_PushInt4    (cstr name, int x, int y, int z, int w);
            void ShaderVar_PushMatrix  (cstr name, Matrix const* m);
            void ShaderVar_PushTex1D   (cstr name, Tex1D* t);
            void ShaderVar_PushTex2D   (cstr name, Tex2D* t);
            void ShaderVar_PushTex3D   (cstr name, Tex3D* t);
            void ShaderVar_PushTexCube (cstr name, TexCube* t);
            void ShaderVar_Pop         (cstr name);
        ]]
    end

    do -- Global Symbol Table
        ShaderVar = {
            PushFloat   = libphx.ShaderVar_PushFloat,
            PushFloat2  = libphx.ShaderVar_PushFloat2,
            PushFloat3  = libphx.ShaderVar_PushFloat3,
            PushFloat4  = libphx.ShaderVar_PushFloat4,
            PushInt     = libphx.ShaderVar_PushInt,
            PushInt2    = libphx.ShaderVar_PushInt2,
            PushInt3    = libphx.ShaderVar_PushInt3,
            PushInt4    = libphx.ShaderVar_PushInt4,
            PushMatrix  = libphx.ShaderVar_PushMatrix,
            PushTex1D   = libphx.ShaderVar_PushTex1D,
            PushTex2D   = libphx.ShaderVar_PushTex2D,
            PushTex3D   = libphx.ShaderVar_PushTex3D,
            PushTexCube = libphx.ShaderVar_PushTexCube,
            Pop         = libphx.ShaderVar_Pop,
        }

        if onDef_ShaderVar then onDef_ShaderVar(ShaderVar, mt) end
        ShaderVar = setmetatable(ShaderVar, mt)
    end

    return ShaderVar
end

return Loader
