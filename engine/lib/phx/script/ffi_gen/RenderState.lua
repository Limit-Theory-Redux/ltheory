-- AUTO GENERATED. DO NOT MODIFY!
-- RenderState -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'RenderState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderState

    do -- C Definitions
        ffi.cdef [[
            void RenderState_PushAllDefaults   (Renderer* r);
            void RenderState_PushBlendMode     (Renderer* r, BlendMode value);
            void RenderState_PushCullFace      (Renderer* r, CullFace value);
            void RenderState_PushDepthTest     (Renderer* r, bool value);
            void RenderState_PushDepthWritable (Renderer* r, bool value);
            void RenderState_PushWireframe     (Renderer* r, bool value);
            void RenderState_PopAll            (Renderer* r);
            void RenderState_PopBlendMode      (Renderer* r);
            void RenderState_PopWireframe      (Renderer* r);
            void RenderState_PopDepthTest      (Renderer* r);
            void RenderState_PopCullFace       (Renderer* r);
            void RenderState_PopDepthWritable  (Renderer* r);
        ]]
    end

    do -- Global Symbol Table
        RenderState = {
            PushAllDefaults   = libphx.RenderState_PushAllDefaults,
            PushBlendMode     = libphx.RenderState_PushBlendMode,
            PushCullFace      = libphx.RenderState_PushCullFace,
            PushDepthTest     = libphx.RenderState_PushDepthTest,
            PushDepthWritable = libphx.RenderState_PushDepthWritable,
            PushWireframe     = libphx.RenderState_PushWireframe,
            PopAll            = libphx.RenderState_PopAll,
            PopBlendMode      = libphx.RenderState_PopBlendMode,
            PopWireframe      = libphx.RenderState_PopWireframe,
            PopDepthTest      = libphx.RenderState_PopDepthTest,
            PopCullFace       = libphx.RenderState_PopCullFace,
            PopDepthWritable  = libphx.RenderState_PopDepthWritable,
        }

        if onDef_RenderState then onDef_RenderState(RenderState, mt) end
        RenderState = setmetatable(RenderState, mt)
    end

    return RenderState
end

return Loader
