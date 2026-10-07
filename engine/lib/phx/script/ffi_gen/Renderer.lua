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
            void        Renderer_Free               (Renderer*);
            bool        Renderer_Sync               (Renderer*);
            uint64      Renderer_StatsDrawCalls     (Renderer*);
            uint64      Renderer_StatsFrameTimeUs   (Renderer*);
            uint64      Renderer_StatsRecvWaitUs    (Renderer*);
            uint64      Renderer_StatsPresentWaitUs (Renderer*);
            uint64      Renderer_StatsFrameCount    (Renderer*);
            uint64      Renderer_StatsCommands      (Renderer*);
            uint64      Renderer_StatsMainWaitUs    (Renderer const*);
            uint64      Renderer_StatsVertices      (Renderer*);
            void        Renderer_SetViewport        (Renderer*, int x, int y, int width, int height);
            void        Renderer_SetScissor         (Renderer*, int x, int y, int width, int height);
            void        Renderer_EnableScissor      (Renderer*, bool enable);
            void        Renderer_SetBlendMode       (Renderer*, BlendMode mode);
            void        Renderer_SetCullFace        (Renderer*, CullFace face);
            void        Renderer_SetDepthTest       (Renderer*, bool enable);
            void        Renderer_SetDepthWritable   (Renderer*, bool enable);
            void        Renderer_SetWireframe       (Renderer*, bool enable);
            void        Renderer_BindShader         (Renderer*, uint32 handle);
            void        Renderer_UnbindShader       (Renderer*);
            void        Renderer_SetUniformInt      (Renderer*, int location, int value);
            void        Renderer_SetUniformFloat    (Renderer*, int location, float value);
            void        Renderer_SetUniformFloat2   (Renderer*, int location, float x, float y);
            void        Renderer_SetUniformFloat3   (Renderer*, int location, float x, float y, float z);
            void        Renderer_SetUniformFloat4   (Renderer*, int location, float x, float y, float z, float w);
            void        Renderer_BindTexture2D      (Renderer*, uint32 slot, uint32 handle);
            void        Renderer_BindTexture3D      (Renderer*, uint32 slot, uint32 handle);
            void        Renderer_BindTextureCube    (Renderer*, uint32 slot, uint32 handle);
            void        Renderer_UnbindTexture      (Renderer*, uint32 slot);
            RenderPass* Renderer_BeginPass          (Renderer*, RenderPassDesc const* desc);
            RenderPass* Renderer_CurrentPass        (Renderer const*);
            void        Renderer_DrawMesh           (Renderer*, uint32 vao, int indexCount);
            void        Renderer_DrawMeshPrimitive  (Renderer*, uint32 vao, int indexCount, CmdPrimitiveType* primitive);
            void        Renderer_Resize             (Renderer*, uint32 width, uint32 height);
            void        Renderer_SwapBuffers        (Renderer*);
            void        Renderer_SetCamera          (Renderer*, Matrix const* view, Matrix const* proj, Vec3f const* starDir);
            void        Renderer_SetEnvironment     (Renderer*, TexCube const* envMap, TexCube const* irMap);
            uint32      Renderer_CreateBindGroup    (Renderer*, BindGroupDesc const* desc);
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
                sync               = libphx.Renderer_Sync,
                statsDrawCalls     = libphx.Renderer_StatsDrawCalls,
                statsFrameTimeUs   = libphx.Renderer_StatsFrameTimeUs,
                statsRecvWaitUs    = libphx.Renderer_StatsRecvWaitUs,
                statsPresentWaitUs = libphx.Renderer_StatsPresentWaitUs,
                statsFrameCount    = libphx.Renderer_StatsFrameCount,
                statsCommands      = libphx.Renderer_StatsCommands,
                statsMainWaitUs    = libphx.Renderer_StatsMainWaitUs,
                statsVertices      = libphx.Renderer_StatsVertices,
                setViewport        = libphx.Renderer_SetViewport,
                setScissor         = libphx.Renderer_SetScissor,
                enableScissor      = libphx.Renderer_EnableScissor,
                setBlendMode       = libphx.Renderer_SetBlendMode,
                setCullFace        = libphx.Renderer_SetCullFace,
                setDepthTest       = libphx.Renderer_SetDepthTest,
                setDepthWritable   = libphx.Renderer_SetDepthWritable,
                setWireframe       = libphx.Renderer_SetWireframe,
                bindShader         = libphx.Renderer_BindShader,
                unbindShader       = libphx.Renderer_UnbindShader,
                setUniformInt      = libphx.Renderer_SetUniformInt,
                setUniformFloat    = libphx.Renderer_SetUniformFloat,
                setUniformFloat2   = libphx.Renderer_SetUniformFloat2,
                setUniformFloat3   = libphx.Renderer_SetUniformFloat3,
                setUniformFloat4   = libphx.Renderer_SetUniformFloat4,
                bindTexture2D      = libphx.Renderer_BindTexture2D,
                bindTexture3D      = libphx.Renderer_BindTexture3D,
                bindTextureCube    = libphx.Renderer_BindTextureCube,
                unbindTexture      = libphx.Renderer_UnbindTexture,
                beginPass          = function(self, desc)
                    local _instance = libphx.Renderer_BeginPass(self, desc)
                    return Core.ManagedObject(_instance, libphx.RenderPass_Free)
                end,
                currentPass        = function(self)
                    local _instance = libphx.Renderer_CurrentPass(self)
                    return Core.ManagedObject(_instance, libphx.RenderPass_Free)
                end,
                drawMesh           = libphx.Renderer_DrawMesh,
                drawMeshPrimitive  = function(self, vao, indexCount, primitive)
                    ffi.gc(primitive, nil)
                    libphx.Renderer_DrawMeshPrimitive(self, vao, indexCount, primitive)
                end,
                resize             = libphx.Renderer_Resize,
                swapBuffers        = libphx.Renderer_SwapBuffers,
                setCamera          = libphx.Renderer_SetCamera,
                setEnvironment     = libphx.Renderer_SetEnvironment,
                createBindGroup    = libphx.Renderer_CreateBindGroup,
            },
        }

        if onDef_Renderer_t then onDef_Renderer_t(t, mt) end
        Renderer_t = ffi.metatype(t, mt)
    end

    return Renderer
end

return Loader
