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
            void              Renderer_setViewport            (Renderer*, int x, int y, int width, int height);
            void              Renderer_setScissor             (Renderer*, int x, int y, int width, int height);
            void              Renderer_enableScissor          (Renderer*, bool enable);
            void              Renderer_setBlendMode           (Renderer*, int mode);
            void              Renderer_setCullFace            (Renderer*, int face);
            void              Renderer_setDepthTest           (Renderer*, bool enable);
            void              Renderer_setDepthWritable       (Renderer*, bool enable);
            void              Renderer_setWireframe           (Renderer*, bool enable);
            void              Renderer_bindShader             (Renderer*, uint32 handle);
            void              Renderer_unbindShader           (Renderer*);
            void              Renderer_setUniformInt          (Renderer*, int location, int value);
            void              Renderer_setUniformFloat        (Renderer*, int location, float value);
            void              Renderer_setUniformFloat2       (Renderer*, int location, float x, float y);
            void              Renderer_setUniformFloat3       (Renderer*, int location, float x, float y, float z);
            void              Renderer_setUniformFloat4       (Renderer*, int location, float x, float y, float z, float w);
            void              Renderer_bindTexture2D          (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_bindTexture3D          (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_bindTextureCube        (Renderer*, uint32 slot, uint32 handle);
            void              Renderer_unbindTexture          (Renderer*, uint32 slot);
            void              Renderer_bindFramebuffer        (Renderer*, uint32 handle);
            void              Renderer_bindDefaultFramebuffer (Renderer*);
            void              Renderer_clearColor             (Renderer*, float r, float g, float b, float a);
            void              Renderer_clearDepth             (Renderer*, float depth);
            void              Renderer_clear                  (Renderer*, float r, float g, float b, float a, float depth);
            void              Renderer_drawMesh               (Renderer*, uint32 vao, int indexCount);
            void              Renderer_drawMeshPrimitive      (Renderer*, uint32 vao, int indexCount, int primitive);
            void              Renderer_drawMeshInstanced      (Renderer*, uint32 vao, int indexCount, int instanceCount);
            void              Renderer_resize                 (Renderer*, uint32 width, uint32 height);
            void              Renderer_swapBuffers            (Renderer*);
            void              Renderer_CreateCameraUBO        (Renderer*);
            void              Renderer_UpdateCameraUBO        (Renderer*, Matrix const* mView, Matrix const* mProj, float eyeX, float eyeY, float eyeZ, float starDirX, float starDirY, float starDirZ);
            void              Renderer_CreateMaterialUBO      (Renderer*);
            void              Renderer_UpdateMaterialUBO      (Renderer*, float r, float g, float b, float a, float metallic, float roughness, float emission);
            void              Renderer_CreateLightUBO         (Renderer*);
            void              Renderer_UpdateLightUBO         (Renderer*, float posX, float posY, float posZ, float radius, float r, float g, float b, float intensity);
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
                setViewport            = libphx.Renderer_setViewport,
                setScissor             = libphx.Renderer_setScissor,
                enableScissor          = libphx.Renderer_enableScissor,
                setBlendMode           = libphx.Renderer_setBlendMode,
                setCullFace            = libphx.Renderer_setCullFace,
                setDepthTest           = libphx.Renderer_setDepthTest,
                setDepthWritable       = libphx.Renderer_setDepthWritable,
                setWireframe           = libphx.Renderer_setWireframe,
                bindShader             = libphx.Renderer_bindShader,
                unbindShader           = libphx.Renderer_unbindShader,
                setUniformInt          = libphx.Renderer_setUniformInt,
                setUniformFloat        = libphx.Renderer_setUniformFloat,
                setUniformFloat2       = libphx.Renderer_setUniformFloat2,
                setUniformFloat3       = libphx.Renderer_setUniformFloat3,
                setUniformFloat4       = libphx.Renderer_setUniformFloat4,
                bindTexture2D          = libphx.Renderer_bindTexture2D,
                bindTexture3D          = libphx.Renderer_bindTexture3D,
                bindTextureCube        = libphx.Renderer_bindTextureCube,
                unbindTexture          = libphx.Renderer_unbindTexture,
                bindFramebuffer        = libphx.Renderer_bindFramebuffer,
                bindDefaultFramebuffer = libphx.Renderer_bindDefaultFramebuffer,
                clearColor             = libphx.Renderer_clearColor,
                clearDepth             = libphx.Renderer_clearDepth,
                clear                  = libphx.Renderer_clear,
                drawMesh               = libphx.Renderer_drawMesh,
                drawMeshPrimitive      = libphx.Renderer_drawMeshPrimitive,
                drawMeshInstanced      = libphx.Renderer_drawMeshInstanced,
                resize                 = libphx.Renderer_resize,
                swapBuffers            = libphx.Renderer_swapBuffers,
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
