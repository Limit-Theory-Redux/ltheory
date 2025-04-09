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
            void RenderState_PushAllDefaults   ();
            void RenderState_PushBlendMode     (BlendMode value);
            void RenderState_PushCullFace      (CullFace value);
            void RenderState_PushDepthTest     (bool value);
            void RenderState_PushDepthWritable (bool value);
            void RenderState_PushWireframe     (bool value);
            void RenderState_PopAll            ();
            void RenderState_PopBlendMode      ();
            void RenderState_PopWireframe      ();
            void RenderState_PopDepthTest      ();
            void RenderState_PopCullFace       ();
            void RenderState_PopDepthWritable  ();
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
