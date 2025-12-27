-- AUTO GENERATED. DO NOT MODIFY!
-- RenderQueue -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RenderQueue {} RenderQueue;
    ]]

    return 1, 'RenderQueue'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderQueue

    do -- C Definitions
        ffi.cdef [[
            void RenderQueue_Free                   (RenderQueue*);
            void RenderQueue_BeginFrame             (RenderQueue const*);
            void RenderQueue_Flush                  (RenderQueue const*);
            bool RenderQueue_Sync                   (RenderQueue const*);
            void RenderQueue_SetViewport            (RenderQueue const*, int x, int y, int width, int height);
            void RenderQueue_SetScissor             (RenderQueue const*, int x, int y, int width, int height);
            void RenderQueue_EnableScissor          (RenderQueue const*, bool enable);
            void RenderQueue_SetBlendMode           (RenderQueue const*, int mode);
            void RenderQueue_SetCullFace            (RenderQueue const*, int face);
            void RenderQueue_SetDepthTest           (RenderQueue const*, bool enable);
            void RenderQueue_SetDepthWritable       (RenderQueue const*, bool enable);
            void RenderQueue_SetWireframe           (RenderQueue const*, bool enable);
            void RenderQueue_BindShader             (RenderQueue const*, uint32 handle);
            void RenderQueue_UnbindShader           (RenderQueue const*);
            void RenderQueue_SetUniformInt          (RenderQueue const*, int location, int value);
            void RenderQueue_SetUniformFloat        (RenderQueue const*, int location, float value);
            void RenderQueue_SetUniformFloat2       (RenderQueue const*, int location, float x, float y);
            void RenderQueue_SetUniformFloat3       (RenderQueue const*, int location, float x, float y, float z);
            void RenderQueue_SetUniformFloat4       (RenderQueue const*, int location, float x, float y, float z, float w);
            void RenderQueue_BindTexture2D          (RenderQueue const*, uint32 slot, uint32 handle);
            void RenderQueue_BindTexture3D          (RenderQueue const*, uint32 slot, uint32 handle);
            void RenderQueue_BindTextureCube        (RenderQueue const*, uint32 slot, uint32 handle);
            void RenderQueue_UnbindTexture          (RenderQueue const*, uint32 slot);
            void RenderQueue_BindFramebuffer        (RenderQueue const*, uint32 handle);
            void RenderQueue_BindDefaultFramebuffer (RenderQueue const*);
            void RenderQueue_ClearColor             (RenderQueue const*, float r, float g, float b, float a);
            void RenderQueue_ClearDepth             (RenderQueue const*, float depth);
            void RenderQueue_Clear                  (RenderQueue const*, float r, float g, float b, float a, float depth);
            void RenderQueue_DrawMesh               (RenderQueue const*, uint32 vao, int indexCount);
            void RenderQueue_DrawMeshPrimitive      (RenderQueue const*, uint32 vao, int indexCount, int primitive);
            void RenderQueue_DrawMeshInstanced      (RenderQueue const*, uint32 vao, int indexCount, int instanceCount);
            void RenderQueue_Resize                 (RenderQueue const*, uint32 width, uint32 height);
            void RenderQueue_SwapBuffers            (RenderQueue const*);
            void RenderQueue_CreateCameraUBO        (RenderQueue const*);
            void RenderQueue_UpdateCameraUBO        (RenderQueue const*, Matrix const* mView, Matrix const* mProj, float eyeX, float eyeY, float eyeZ, float starDirX, float starDirY, float starDirZ);
            void RenderQueue_CreateMaterialUBO      (RenderQueue const*);
            void RenderQueue_UpdateMaterialUBO      (RenderQueue const*, float r, float g, float b, float a, float metallic, float roughness, float emission);
            void RenderQueue_CreateLightUBO         (RenderQueue const*);
            void RenderQueue_UpdateLightUBO         (RenderQueue const*, float posX, float posY, float posZ, float radius, float r, float g, float b, float intensity);
        ]]
    end

    do -- Global Symbol Table
        RenderQueue = {}

        if onDef_RenderQueue then onDef_RenderQueue(RenderQueue, mt) end
        RenderQueue = setmetatable(RenderQueue, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RenderQueue')
        local mt = {
            __index = {
                beginFrame             = libphx.RenderQueue_BeginFrame,
                flush                  = libphx.RenderQueue_Flush,
                sync                   = libphx.RenderQueue_Sync,
                setViewport            = libphx.RenderQueue_SetViewport,
                setScissor             = libphx.RenderQueue_SetScissor,
                enableScissor          = libphx.RenderQueue_EnableScissor,
                setBlendMode           = libphx.RenderQueue_SetBlendMode,
                setCullFace            = libphx.RenderQueue_SetCullFace,
                setDepthTest           = libphx.RenderQueue_SetDepthTest,
                setDepthWritable       = libphx.RenderQueue_SetDepthWritable,
                setWireframe           = libphx.RenderQueue_SetWireframe,
                bindShader             = libphx.RenderQueue_BindShader,
                unbindShader           = libphx.RenderQueue_UnbindShader,
                setUniformInt          = libphx.RenderQueue_SetUniformInt,
                setUniformFloat        = libphx.RenderQueue_SetUniformFloat,
                setUniformFloat2       = libphx.RenderQueue_SetUniformFloat2,
                setUniformFloat3       = libphx.RenderQueue_SetUniformFloat3,
                setUniformFloat4       = libphx.RenderQueue_SetUniformFloat4,
                bindTexture2D          = libphx.RenderQueue_BindTexture2D,
                bindTexture3D          = libphx.RenderQueue_BindTexture3D,
                bindTextureCube        = libphx.RenderQueue_BindTextureCube,
                unbindTexture          = libphx.RenderQueue_UnbindTexture,
                bindFramebuffer        = libphx.RenderQueue_BindFramebuffer,
                bindDefaultFramebuffer = libphx.RenderQueue_BindDefaultFramebuffer,
                clearColor             = libphx.RenderQueue_ClearColor,
                clearDepth             = libphx.RenderQueue_ClearDepth,
                clear                  = libphx.RenderQueue_Clear,
                drawMesh               = libphx.RenderQueue_DrawMesh,
                drawMeshPrimitive      = libphx.RenderQueue_DrawMeshPrimitive,
                drawMeshInstanced      = libphx.RenderQueue_DrawMeshInstanced,
                resize                 = libphx.RenderQueue_Resize,
                swapBuffers            = libphx.RenderQueue_SwapBuffers,
                createCameraUBO        = libphx.RenderQueue_CreateCameraUBO,
                updateCameraUBO        = libphx.RenderQueue_UpdateCameraUBO,
                createMaterialUBO      = libphx.RenderQueue_CreateMaterialUBO,
                updateMaterialUBO      = libphx.RenderQueue_UpdateMaterialUBO,
                createLightUBO         = libphx.RenderQueue_CreateLightUBO,
                updateLightUBO         = libphx.RenderQueue_UpdateLightUBO,
            },
        }

        if onDef_RenderQueue_t then onDef_RenderQueue_t(t, mt) end
        RenderQueue_t = ffi.metatype(t, mt)
    end

    return RenderQueue
end

return Loader
