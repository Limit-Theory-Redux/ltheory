-- AUTO GENERATED. DO NOT MODIFY!
-- RenderPass ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RenderPass {} RenderPass;
    ]]

    return 1, 'RenderPass'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderPass

    do -- C Definitions
        ffi.cdef [[
            void RenderPass_Free           (RenderPass*);
            void RenderPass_Finish         (RenderPass*, Renderer* r);
            void RenderPass_SetPipeline    (RenderPass const*, Renderer* r, uint32 pipeline);
            void RenderPass_SetInput       (RenderPass const*, Renderer* r, int slot, TexView const* view, uint32 sampler);
            void RenderPass_ClearInput     (RenderPass const*, Renderer* r, int slot);
            void RenderPass_SetBindGroup   (RenderPass const*, Renderer* r, int group, uint32 bindGroup);
            void RenderPass_DrawMesh       (RenderPass const*, Renderer* r, Mesh* mesh);
            void RenderPass_DrawFullscreen (RenderPass const*, Renderer* r);
            void RenderPass_SetViewport    (RenderPass const*, Renderer* r, int x, int y, int width, int height);
            void RenderPass_SetScissor     (RenderPass const*, Renderer* r, int x, int y, int width, int height);
            void RenderPass_ClearScissor   (RenderPass const*, Renderer* r);
            void RenderPass_SetUiTransform (RenderPass const*, Renderer* r, Matrix const* transform);
        ]]
    end

    do -- Global Symbol Table
        RenderPass = {}

        if onDef_RenderPass then onDef_RenderPass(RenderPass, mt) end
        RenderPass = setmetatable(RenderPass, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RenderPass')
        local mt = {
            __index = {
                finish         = libphx.RenderPass_Finish,
                setPipeline    = libphx.RenderPass_SetPipeline,
                setInput       = libphx.RenderPass_SetInput,
                clearInput     = libphx.RenderPass_ClearInput,
                setBindGroup   = libphx.RenderPass_SetBindGroup,
                drawMesh       = libphx.RenderPass_DrawMesh,
                drawFullscreen = libphx.RenderPass_DrawFullscreen,
                setViewport    = libphx.RenderPass_SetViewport,
                setScissor     = libphx.RenderPass_SetScissor,
                clearScissor   = libphx.RenderPass_ClearScissor,
                setUiTransform = libphx.RenderPass_SetUiTransform,
            },
        }

        if onDef_RenderPass_t then onDef_RenderPass_t(t, mt) end
        RenderPass_t = ffi.metatype(t, mt)
    end

    return RenderPass
end

return Loader
