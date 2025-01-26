-- AUTO GENERATED. DO NOT MODIFY!
-- ShaderState -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct ShaderState {} ShaderState;
    ]]

    return 1, 'ShaderState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ShaderState

    do -- C Definitions
        ffi.cdef [[
            void         ShaderState_Free           (ShaderState*);
            ShaderState* ShaderState_Create         (Shader const* shader);
            ShaderState* ShaderState_FromShaderLoad (cstr vsName, cstr fsName);
            void         ShaderState_SetFloat       (ShaderState*, cstr name, float x);
            void         ShaderState_SetFloat2      (ShaderState*, cstr name, float x, float y);
            void         ShaderState_SetFloat3      (ShaderState*, cstr name, float x, float y, float z);
            void         ShaderState_SetFloat4      (ShaderState*, cstr name, float x, float y, float z, float w);
            void         ShaderState_SetInt         (ShaderState*, cstr name, int x);
            void         ShaderState_SetInt2        (ShaderState*, cstr name, int x, int y);
            void         ShaderState_SetInt3        (ShaderState*, cstr name, int x, int y, int z);
            void         ShaderState_SetInt4        (ShaderState*, cstr name, int x, int y, int z, int w);
            void         ShaderState_SetMatrix      (ShaderState*, cstr name, Matrix const* m);
            void         ShaderState_SetTex1D       (ShaderState*, cstr name, Tex1D* t);
            void         ShaderState_SetTex2D       (ShaderState*, cstr name, Tex2D* t);
            void         ShaderState_SetTex3D       (ShaderState*, cstr name, Tex3D* t);
            void         ShaderState_SetTexCube     (ShaderState*, cstr name, TexCube* t);
            void         ShaderState_Start          (ShaderState*);
            void         ShaderState_Stop           (ShaderState*);
            Shader*      ShaderState_Shader         (ShaderState*);
        ]]
    end

    do -- Global Symbol Table
        ShaderState = {
            Create         = function(shader)
                local _instance = libphx.ShaderState_Create(shader)
                return Core.ManagedObject(_instance, libphx.ShaderState_Free)
            end,
            FromShaderLoad = function(vsName, fsName)
                local _instance = libphx.ShaderState_FromShaderLoad(vsName, fsName)
                return Core.ManagedObject(_instance, libphx.ShaderState_Free)
            end,
        }

        if onDef_ShaderState then onDef_ShaderState(ShaderState, mt) end
        ShaderState = setmetatable(ShaderState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('ShaderState')
        local mt = {
            __index = {
                setFloat   = libphx.ShaderState_SetFloat,
                setFloat2  = libphx.ShaderState_SetFloat2,
                setFloat3  = libphx.ShaderState_SetFloat3,
                setFloat4  = libphx.ShaderState_SetFloat4,
                setInt     = libphx.ShaderState_SetInt,
                setInt2    = libphx.ShaderState_SetInt2,
                setInt3    = libphx.ShaderState_SetInt3,
                setInt4    = libphx.ShaderState_SetInt4,
                setMatrix  = libphx.ShaderState_SetMatrix,
                setTex1D   = libphx.ShaderState_SetTex1D,
                setTex2D   = libphx.ShaderState_SetTex2D,
                setTex3D   = libphx.ShaderState_SetTex3D,
                setTexCube = libphx.ShaderState_SetTexCube,
                start      = libphx.ShaderState_Start,
                stop       = libphx.ShaderState_Stop,
                shader     = libphx.ShaderState_Shader,
            },
        }

        if onDef_ShaderState_t then onDef_ShaderState_t(t, mt) end
        ShaderState_t = ffi.metatype(t, mt)
    end

    return ShaderState
end

return Loader
