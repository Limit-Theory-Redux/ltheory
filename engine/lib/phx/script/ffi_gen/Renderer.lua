-- AUTO GENERATED. DO NOT MODIFY!
-- Renderer --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Renderer {} Renderer;
    ]]

    return 1, 'Renderer'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Renderer

    do -- C Definitions
        ffi.cdef [[
            void              Renderer_Free                   (Renderer*);
            void              Renderer_BeginFrame             (Renderer*);
            void              Renderer_Flush                  (Renderer*);
            bool              Renderer_Sync                   (Renderer*);
            void              Renderer_BeginBatch             (Renderer*, float const* view, uint64 view_size, float const* projection, uint64 projection_size, float eyeX, float eyeY, float eyeZ);
            void              Renderer_AddEntity              (Renderer*, float const* transform, uint64 transform_size, float boundsCenterX, float boundsCenterY, float boundsCenterZ, float boundsRadius, uint32 meshVao, int indexCount, uint32 shaderHandle, uint32 sortKey);
            void              Renderer_FlushBatch             (Renderer*);
            BatchStats const* Renderer_GetBatchStats          (Renderer const*);
            void              Renderer_SetViewport            (Renderer const*, int x, int y, int width, int height);
            void              Renderer_SetScissor             (Renderer const*, int x, int y, int width, int height);
            void              Renderer_EnableScissor          (Renderer const*, bool enable);
            void              Renderer_SetBlendMode           (Renderer const*, int mode);
            void              Renderer_SetCullFace            (Renderer const*, int face);
            void              Renderer_SetDepthTest           (Renderer const*, bool enable);
            void              Renderer_SetDepthWritable       (Renderer const*, bool enable);
            void              Renderer_SetWireframe           (Renderer const*, bool enable);
            void              Renderer_BindShader             (Renderer const*, uint32 handle);
            void              Renderer_UnbindShader           (Renderer const*);
            void              Renderer_SetUniformInt          (Renderer const*, int location, int value);
            void              Renderer_SetUniformFloat        (Renderer const*, int location, float value);
            void              Renderer_SetUniformFloat2       (Renderer const*, int location, float x, float y);
            void              Renderer_SetUniformFloat3       (Renderer const*, int location, float x, float y, float z);
            void              Renderer_SetUniformFloat4       (Renderer const*, int location, float x, float y, float z, float w);
            void              Renderer_BindTexture2D          (Renderer const*, uint32 slot, uint32 handle);
            void              Renderer_BindTexture3D          (Renderer const*, uint32 slot, uint32 handle);
            void              Renderer_BindTextureCube        (Renderer const*, uint32 slot, uint32 handle);
            void              Renderer_UnbindTexture          (Renderer const*, uint32 slot);
            void              Renderer_BindFramebuffer        (Renderer const*, uint32 handle);
            void              Renderer_BindDefaultFramebuffer (Renderer const*);
            void              Renderer_ClearColor             (Renderer const*, float r, float g, float b, float a);
            void              Renderer_ClearDepth             (Renderer const*, float depth);
            void              Renderer_Clear                  (Renderer const*, float r, float g, float b, float a, float depth);
            void              Renderer_DrawMesh               (Renderer const*, uint32 vao, int indexCount);
            void              Renderer_DrawMeshPrimitive      (Renderer const*, uint32 vao, int indexCount, int primitive);
            void              Renderer_DrawMeshInstanced      (Renderer const*, uint32 vao, int indexCount, int instanceCount);
            void              Renderer_Resize                 (Renderer const*, uint32 width, uint32 height);
            void              Renderer_SwapBuffers            (Renderer const*);
            void              Renderer_CreateCameraUBO        (Renderer const*);
            void              Renderer_UpdateCameraUBO        (Renderer const*, Matrix const* mView, Matrix const* mProj, float eyeX, float eyeY, float eyeZ, float starDirX, float starDirY, float starDirZ);
            void              Renderer_CreateMaterialUBO      (Renderer const*);
            void              Renderer_UpdateMaterialUBO      (Renderer const*, float r, float g, float b, float a, float metallic, float roughness, float emission);
            void              Renderer_CreateLightUBO         (Renderer const*);
            void              Renderer_UpdateLightUBO         (Renderer const*, float posX, float posY, float posZ, float radius, float r, float g, float b, float intensity);
        ]]
    end

    do -- Global Symbol Table
        Renderer = {}

        if onDef_Renderer then onDef_Renderer(Renderer, mt) end
        Renderer = setmetatable(Renderer, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Renderer')
        local mt = {
            __index = {
                beginFrame             = libphx.Renderer_BeginFrame,
                flush                  = libphx.Renderer_Flush,
                sync                   = libphx.Renderer_Sync,
                beginBatch             = libphx.Renderer_BeginBatch,
                addEntity              = libphx.Renderer_AddEntity,
                flushBatch             = libphx.Renderer_FlushBatch,
                getBatchStats          = libphx.Renderer_GetBatchStats,
                setViewport            = libphx.Renderer_SetViewport,
                setScissor             = libphx.Renderer_SetScissor,
                enableScissor          = libphx.Renderer_EnableScissor,
                setBlendMode           = libphx.Renderer_SetBlendMode,
                setCullFace            = libphx.Renderer_SetCullFace,
                setDepthTest           = libphx.Renderer_SetDepthTest,
                setDepthWritable       = libphx.Renderer_SetDepthWritable,
                setWireframe           = libphx.Renderer_SetWireframe,
                bindShader             = libphx.Renderer_BindShader,
                unbindShader           = libphx.Renderer_UnbindShader,
                setUniformInt          = libphx.Renderer_SetUniformInt,
                setUniformFloat        = libphx.Renderer_SetUniformFloat,
                setUniformFloat2       = libphx.Renderer_SetUniformFloat2,
                setUniformFloat3       = libphx.Renderer_SetUniformFloat3,
                setUniformFloat4       = libphx.Renderer_SetUniformFloat4,
                bindTexture2D          = libphx.Renderer_BindTexture2D,
                bindTexture3D          = libphx.Renderer_BindTexture3D,
                bindTextureCube        = libphx.Renderer_BindTextureCube,
                unbindTexture          = libphx.Renderer_UnbindTexture,
                bindFramebuffer        = libphx.Renderer_BindFramebuffer,
                bindDefaultFramebuffer = libphx.Renderer_BindDefaultFramebuffer,
                clearColor             = libphx.Renderer_ClearColor,
                clearDepth             = libphx.Renderer_ClearDepth,
                clear                  = libphx.Renderer_Clear,
                drawMesh               = libphx.Renderer_DrawMesh,
                drawMeshPrimitive      = libphx.Renderer_DrawMeshPrimitive,
                drawMeshInstanced      = libphx.Renderer_DrawMeshInstanced,
                resize                 = libphx.Renderer_Resize,
                swapBuffers            = libphx.Renderer_SwapBuffers,
                createCameraUBO        = libphx.Renderer_CreateCameraUBO,
                updateCameraUBO        = libphx.Renderer_UpdateCameraUBO,
                createMaterialUBO      = libphx.Renderer_CreateMaterialUBO,
                updateMaterialUBO      = libphx.Renderer_UpdateMaterialUBO,
                createLightUBO         = libphx.Renderer_CreateLightUBO,
                updateLightUBO         = libphx.Renderer_UpdateLightUBO,
            },
        }

        if onDef_Renderer_t then onDef_Renderer_t(t, mt) end
        Renderer_t = ffi.metatype(t, mt)
    end

    return Renderer
end

return Loader
