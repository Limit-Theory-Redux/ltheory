-- Shader ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Shader {} Shader;
    ]]

    return 1, 'Shader'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Shader

    do -- C Definitions
        ffi.cdef [[
            void         Shader_Free          (Shader*);
            Shader*      Shader_Create        (cstr vs, cstr fs);
            Shader*      Shader_Load          (cstr vsName, cstr fsName);
            Shader*      Shader_Clone         (Shader const*);
            ShaderState* Shader_ToShaderState (Shader const*);
            uint32       Shader_GetHandle     (Shader const*);
            int          Shader_GetVariable   (Shader const*, cstr name);
            bool         Shader_HasVariable   (Shader const*, cstr name);
            void         Shader_Start         (Shader*);
            void         Shader_Stop          (Shader const*);
            void         Shader_ResetTexIndex ();
            void         Shader_SetFloat      (cstr name, float value);
            void         Shader_ISetFloat     (int index, float value);
            void         Shader_SetFloat2     (cstr name, float x, float y);
            void         Shader_ISetFloat2    (int index, float x, float y);
            void         Shader_SetFloat3     (cstr name, float x, float y, float z);
            void         Shader_ISetFloat3    (int index, float x, float y, float z);
            void         Shader_SetFloat4     (cstr name, float x, float y, float z, float w);
            void         Shader_ISetFloat4    (int index, float x, float y, float z, float w);
            void         Shader_SetInt        (cstr name, int value);
            void         Shader_ISetInt       (int index, int value);
            void         Shader_SetInt2       (cstr name, int x, int y);
            void         Shader_ISetInt2      (int index, int x, int y);
            void         Shader_SetInt3       (cstr name, int x, int y, int z);
            void         Shader_ISetInt3      (int index, int x, int y, int z);
            void         Shader_SetInt4       (cstr name, int x, int y, int z, int w);
            void         Shader_ISetInt4      (int index, int x, int y, int z, int w);
            void         Shader_SetMatrix     (cstr name, Matrix const* value);
            void         Shader_ISetMatrix    (int index, Matrix const* value);
            void         Shader_SetMatrixT    (cstr name, Matrix const* value);
            void         Shader_ISetMatrixT   (int index, Matrix const* value);
            void         Shader_SetTex1D      (cstr name, Tex1D* value);
            void         Shader_ISetTex1D     (int index, Tex1D* value);
            void         Shader_SetTex2D      (cstr name, Tex2D* value);
            void         Shader_ISetTex2D     (int index, Tex2D* value);
            void         Shader_SetTex3D      (cstr name, Tex3D* value);
            void         Shader_ISetTex3D     (int index, Tex3D* value);
            void         Shader_SetTexCube    (cstr name, TexCube* value);
            void         Shader_ISetTexCube   (int index, TexCube* value);
        ]]
    end

    do -- Global Symbol Table
        Shader = {
            Create        = function(...)
                local instance = libphx.Shader_Create(...)
                return Core.ManagedObject(instance, libphx.Shader_Free)
            end,
            Load          = function(...)
                local instance = libphx.Shader_Load(...)
                return Core.ManagedObject(instance, libphx.Shader_Free)
            end,
            ResetTexIndex = libphx.Shader_ResetTexIndex,
            SetFloat      = libphx.Shader_SetFloat,
            ISetFloat     = libphx.Shader_ISetFloat,
            SetFloat2     = libphx.Shader_SetFloat2,
            ISetFloat2    = libphx.Shader_ISetFloat2,
            SetFloat3     = libphx.Shader_SetFloat3,
            ISetFloat3    = libphx.Shader_ISetFloat3,
            SetFloat4     = libphx.Shader_SetFloat4,
            ISetFloat4    = libphx.Shader_ISetFloat4,
            SetInt        = libphx.Shader_SetInt,
            ISetInt       = libphx.Shader_ISetInt,
            SetInt2       = libphx.Shader_SetInt2,
            ISetInt2      = libphx.Shader_ISetInt2,
            SetInt3       = libphx.Shader_SetInt3,
            ISetInt3      = libphx.Shader_ISetInt3,
            SetInt4       = libphx.Shader_SetInt4,
            ISetInt4      = libphx.Shader_ISetInt4,
            SetMatrix     = libphx.Shader_SetMatrix,
            ISetMatrix    = libphx.Shader_ISetMatrix,
            SetMatrixT    = libphx.Shader_SetMatrixT,
            ISetMatrixT   = libphx.Shader_ISetMatrixT,
            SetTex1D      = libphx.Shader_SetTex1D,
            ISetTex1D     = libphx.Shader_ISetTex1D,
            SetTex2D      = libphx.Shader_SetTex2D,
            ISetTex2D     = libphx.Shader_ISetTex2D,
            SetTex3D      = libphx.Shader_SetTex3D,
            ISetTex3D     = libphx.Shader_ISetTex3D,
            SetTexCube    = libphx.Shader_SetTexCube,
            ISetTexCube   = libphx.Shader_ISetTexCube,
        }

        if onDef_Shader then onDef_Shader(Shader, mt) end
        Shader = setmetatable(Shader, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Shader')
        local mt = {
            __index = {
                clone         = function(...)
                    local instance = libphx.Shader_Clone(...)
                    return Core.ManagedObject(instance, libphx.Shader_Free)
                end,
                toShaderState = function(...)
                    local instance = libphx.Shader_ToShaderState(...)
                    return Core.ManagedObject(instance, libphx.ShaderState_Free)
                end,
                getHandle     = libphx.Shader_GetHandle,
                getVariable   = libphx.Shader_GetVariable,
                hasVariable   = libphx.Shader_HasVariable,
                start         = libphx.Shader_Start,
                stop          = libphx.Shader_Stop,
            },
        }

        if onDef_Shader_t then onDef_Shader_t(t, mt) end
        Shader_t = ffi.metatype(t, mt)
    end

    return Shader
end

return Loader
