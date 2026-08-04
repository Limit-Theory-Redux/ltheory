-- AUTO GENERATED. DO NOT MODIFY!
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
            bool         Shader_Reload        (Shader*);
            cstr         Shader_Name          (Shader const*);
            Shader*      Shader_Clone         (Shader const*);
            ShaderState* Shader_ToShaderState (Shader const*);
            uint32       Shader_GetHandle     (Shader const*);
            int          Shader_GetVariable   (Shader const*, cstr name);
            bool         Shader_HasVariable   (Shader const*, cstr name);
            void         Shader_ResetTexIndex (Shader*);
            void         Shader_SetFloat      (Shader*, Renderer* r, cstr name, float value);
            void         Shader_ISetFloat     (Shader*, Renderer* r, int index, float value);
            void         Shader_SetFloat2     (Shader*, Renderer* r, cstr name, float x, float y);
            void         Shader_ISetFloat2    (Shader*, Renderer* r, int index, float x, float y);
            void         Shader_SetFloat3     (Shader*, Renderer* r, cstr name, float x, float y, float z);
            void         Shader_ISetFloat3    (Shader*, Renderer* r, int index, float x, float y, float z);
            void         Shader_SetFloat4     (Shader*, Renderer* r, cstr name, float x, float y, float z, float w);
            void         Shader_ISetFloat4    (Shader*, Renderer* r, int index, float x, float y, float z, float w);
            void         Shader_SetInt        (Shader*, Renderer* r, cstr name, int value);
            void         Shader_ISetInt       (Shader*, Renderer* r, int index, int value);
            void         Shader_SetInt2       (Shader*, Renderer* r, cstr name, int x, int y);
            void         Shader_ISetInt2      (Shader*, Renderer* r, int index, int x, int y);
            void         Shader_SetInt3       (Shader*, Renderer* r, cstr name, int x, int y, int z);
            void         Shader_ISetInt3      (Shader*, Renderer* r, int index, int x, int y, int z);
            void         Shader_SetInt4       (Shader*, Renderer* r, cstr name, int x, int y, int z, int w);
            void         Shader_ISetInt4      (Shader*, Renderer* r, int index, int x, int y, int z, int w);
            void         Shader_SetMatrix     (Shader*, Renderer* r, cstr name, Matrix const* value);
            void         Shader_ISetMatrix    (Shader*, Renderer* r, int index, Matrix const* value);
            void         Shader_SetMatrixT    (Shader*, Renderer* r, cstr name, Matrix const* value);
            void         Shader_ISetMatrixT   (Shader*, Renderer* r, int index, Matrix const* value);
            void         Shader_SetTex1D      (Shader*, Renderer* r, cstr name, Tex1D* value);
            void         Shader_ISetTex1D     (Shader*, Renderer* r, int index, Tex1D* value);
            void         Shader_SetTex2D      (Shader*, Renderer* r, cstr name, Tex2D const* value);
            void         Shader_ISetTex2D     (Shader*, Renderer* r, int index, Tex2D* value);
            void         Shader_SetTex3D      (Shader*, Renderer* r, cstr name, Tex3D* value);
            void         Shader_ISetTex3D     (Shader*, Renderer* r, int index, Tex3D* value);
            void         Shader_SetTexCube    (Shader*, Renderer* r, cstr name, TexCube* value);
            void         Shader_ISetTexCube   (Shader*, Renderer* r, int index, TexCube* value);
            void         Shader_Start         (Shader*, Renderer* r);
            void         Shader_Stop          (Shader const*);
        ]]
    end

    do -- Global Symbol Table
        Shader = {
            Create        = function(vs, fs)
                local _instance = libphx.Shader_Create(vs, fs)
                return Core.ManagedObject(_instance, libphx.Shader_Free)
            end,
            Load          = function(vsName, fsName)
                local _instance = libphx.Shader_Load(vsName, fsName)
                return Core.ManagedObject(_instance, libphx.Shader_Free)
            end,
        }

        if onDef_Shader then onDef_Shader(Shader, mt) end
        Shader = setmetatable(Shader, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Shader')
        local mt = {
            __index = {
                reload        = libphx.Shader_Reload,
                name          = libphx.Shader_Name,
                clone         = function(self)
                    local _instance = libphx.Shader_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Shader_Free)
                end,
                toShaderState = function(self)
                    local _instance = libphx.Shader_ToShaderState(self)
                    return Core.ManagedObject(_instance, libphx.ShaderState_Free)
                end,
                getHandle     = libphx.Shader_GetHandle,
                getVariable   = libphx.Shader_GetVariable,
                hasVariable   = libphx.Shader_HasVariable,
                resetTexIndex = libphx.Shader_ResetTexIndex,
                setFloat      = libphx.Shader_SetFloat,
                iSetFloat     = libphx.Shader_ISetFloat,
                setFloat2     = libphx.Shader_SetFloat2,
                iSetFloat2    = libphx.Shader_ISetFloat2,
                setFloat3     = libphx.Shader_SetFloat3,
                iSetFloat3    = libphx.Shader_ISetFloat3,
                setFloat4     = libphx.Shader_SetFloat4,
                iSetFloat4    = libphx.Shader_ISetFloat4,
                setInt        = libphx.Shader_SetInt,
                iSetInt       = libphx.Shader_ISetInt,
                setInt2       = libphx.Shader_SetInt2,
                iSetInt2      = libphx.Shader_ISetInt2,
                setInt3       = libphx.Shader_SetInt3,
                iSetInt3      = libphx.Shader_ISetInt3,
                setInt4       = libphx.Shader_SetInt4,
                iSetInt4      = libphx.Shader_ISetInt4,
                setMatrix     = libphx.Shader_SetMatrix,
                iSetMatrix    = libphx.Shader_ISetMatrix,
                setMatrixT    = libphx.Shader_SetMatrixT,
                iSetMatrixT   = libphx.Shader_ISetMatrixT,
                setTex1D      = libphx.Shader_SetTex1D,
                iSetTex1D     = libphx.Shader_ISetTex1D,
                setTex2D      = libphx.Shader_SetTex2D,
                iSetTex2D     = libphx.Shader_ISetTex2D,
                setTex3D      = libphx.Shader_SetTex3D,
                iSetTex3D     = libphx.Shader_ISetTex3D,
                setTexCube    = libphx.Shader_SetTexCube,
                iSetTexCube   = libphx.Shader_ISetTexCube,
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
