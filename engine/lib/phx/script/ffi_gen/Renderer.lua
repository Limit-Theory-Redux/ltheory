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
            void            Renderer_Free               (Renderer*);
            bool            Renderer_Sync               (Renderer*);
            void            Renderer_GpuFinish          (Renderer*);
            uint64          Renderer_StatsDrawCalls     (Renderer*);
            uint64          Renderer_StatsFrameTimeUs   (Renderer*);
            uint64          Renderer_StatsRecvWaitUs    (Renderer*);
            uint64          Renderer_StatsPresentWaitUs (Renderer*);
            uint64          Renderer_StatsFrameCount    (Renderer*);
            uint64          Renderer_StatsCommands      (Renderer*);
            uint64          Renderer_StatsMainWaitUs    (Renderer const*);
            uint64          Renderer_StatsVertices      (Renderer*);
            Bytes*          Renderer_ReadSync           (Renderer*, TexView const* view, int x, int y, int w, int h, TexFormat fmt);
            ReadbackTicket* Renderer_ReadAsync          (Renderer*, TexView const* view, int x, int y, int w, int h, TexFormat fmt);
            RenderPass*     Renderer_BeginPass          (Renderer*, RenderPassDesc const* desc);
            RenderPass*     Renderer_CurrentPass        (Renderer const*);
            void            Renderer_Resize             (Renderer*, uint32 width, uint32 height);
            void            Renderer_SwapBuffers        (Renderer*);
            void            Renderer_SetCamera          (Renderer*, Matrix const* view, Matrix const* proj, Vec3f const* starDir);
            void            Renderer_SetEnvironment     (Renderer*, TexCube const* envMap, TexCube const* irMap);
            uint32          Renderer_CreateBindGroup    (Renderer*, BindGroupDesc const* desc);
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
                gpuFinish          = libphx.Renderer_GpuFinish,
                statsDrawCalls     = libphx.Renderer_StatsDrawCalls,
                statsFrameTimeUs   = libphx.Renderer_StatsFrameTimeUs,
                statsRecvWaitUs    = libphx.Renderer_StatsRecvWaitUs,
                statsPresentWaitUs = libphx.Renderer_StatsPresentWaitUs,
                statsFrameCount    = libphx.Renderer_StatsFrameCount,
                statsCommands      = libphx.Renderer_StatsCommands,
                statsMainWaitUs    = libphx.Renderer_StatsMainWaitUs,
                statsVertices      = libphx.Renderer_StatsVertices,
                readSync           = function(self, view, x, y, w, h, fmt)
                    local _instance = libphx.Renderer_ReadSync(self, view, x, y, w, h, fmt)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                readAsync          = function(self, view, x, y, w, h, fmt)
                    local _instance = libphx.Renderer_ReadAsync(self, view, x, y, w, h, fmt)
                    return Core.ManagedObject(_instance, libphx.ReadbackTicket_Free)
                end,
                beginPass          = function(self, desc)
                    local _instance = libphx.Renderer_BeginPass(self, desc)
                    return Core.ManagedObject(_instance, libphx.RenderPass_Free)
                end,
                currentPass        = function(self)
                    local _instance = libphx.Renderer_CurrentPass(self)
                    return Core.ManagedObject(_instance, libphx.RenderPass_Free)
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
