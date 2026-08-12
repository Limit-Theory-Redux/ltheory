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
            void              Renderer_AddEntity              (Renderer*, float const* transform, uint64 transform_size, float boundsCenterX, float boundsCenterY, float boundsCenterZ, float boundsRadius, uint64 meshId, int indexCount, uint64 shaderId, uint32 sortKey);
            void              Renderer_FlushBatch             (Renderer*);
            BatchStats const* Renderer_GetBatchStats          (Renderer const*);
            void              Renderer_SetViewport            (Renderer*, int x, int y, int width, int height);
            void              Renderer_SetScissor             (Renderer*, int x, int y, int width, int height);
            void              Renderer_EnableScissor          (Renderer*, bool enable);
            void              Renderer_SetBlendMode           (Renderer*, BlendMode mode);
            void              Renderer_SetCullFace            (Renderer*, CullFace face);
            void              Renderer_SetDepthTest           (Renderer*, bool enable);
            void              Renderer_SetDepthWritable       (Renderer*, bool enable);
            void              Renderer_SetWireframe           (Renderer*, bool enable);
            void              Renderer_BindShader             (Renderer*, uint32 handle);
            void              Renderer_UnbindShader           (Renderer*);
            void              Renderer_SetUniformInt          (Renderer*, int location, int value);
            void              Renderer_SetUniformFloat        (Renderer*, int location, float value);
            void              Renderer_SetUniformFloat2       (Renderer*, int location, float x, float y);
            void              Renderer_SetUniformFloat3       (Renderer*, int location, float x, float y, float z);
            void              Renderer_SetUniformFloat4       (Renderer*, int location, float x, float y, float z, float w);
            void              Renderer_BindTexture2D          (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_BindTexture3D          (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_BindTextureCube        (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_UnbindTexture          (Renderer*, uint32 slot);
            void              Renderer_BindFramebuffer        (Renderer*, uint32 handle);
            void              Renderer_BindDefaultFramebuffer (Renderer*);
            void              Renderer_ClearColor             (Renderer*, float r, float g, float b, float a);
            void              Renderer_ClearDepth             (Renderer*, float depth);
            void              Renderer_Clear                  (Renderer*, float r, float g, float b, float a, float depth);
            void              Renderer_DrawMesh               (Renderer*, uint32 vao, int indexCount);
            void              Renderer_DrawMeshPrimitive      (Renderer*, uint32 vao, int indexCount, CmdPrimitiveType* primitive);
            void              Renderer_DrawMeshInstanced      (Renderer*, uint32 vao, int indexCount, int instanceCount);
            void              Renderer_Resize                 (Renderer*, uint32 width, uint32 height);
            void              Renderer_SwapBuffers            (Renderer*);
            void              Renderer_CreateCameraUbo        (Renderer*);
            void              Renderer_UpdateCameraUbo        (Renderer*, Matrix const* mView, Matrix const* mProj, float eyeX, float eyeY, float eyeZ, float starDirX, float starDirY, float starDirZ);
            void              Renderer_CreateMaterialUbo      (Renderer*);
            void              Renderer_UpdateMaterialUbo      (Renderer*, float r, float g, float b, float a, float metallic, float roughness, float emission);
            void              Renderer_CreateLightUbo         (Renderer*);
            void              Renderer_UpdateLightUbo         (Renderer*, float posX, float posY, float posZ, float radius, float r, float g, float b, float intensity);
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
                drawMeshPrimitive      = function(self, vao, indexCount, primitive)
                    ffi.gc(primitive, nil)
                    libphx.Renderer_DrawMeshPrimitive(self, vao, indexCount, primitive)
                end,
                drawMeshInstanced      = libphx.Renderer_DrawMeshInstanced,
                resize                 = libphx.Renderer_Resize,
                swapBuffers            = libphx.Renderer_SwapBuffers,
                createCameraUbo        = libphx.Renderer_CreateCameraUbo,
                updateCameraUbo        = libphx.Renderer_UpdateCameraUbo,
                createMaterialUbo      = libphx.Renderer_CreateMaterialUbo,
                updateMaterialUbo      = libphx.Renderer_UpdateMaterialUbo,
                createLightUbo         = libphx.Renderer_CreateLightUbo,
                updateLightUbo         = libphx.Renderer_UpdateLightUbo,
            },
        }

        if onDef_Renderer_t then onDef_Renderer_t(t, mt) end
        Renderer_t = ffi.metatype(t, mt)
    end

    return Renderer
end

return Loader
