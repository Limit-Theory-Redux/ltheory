local libphx = require('libphx').lib

-- RenderState's C functions all take the current Renderer as their first
-- argument now (see ai/multithreaded_rendering.md); inject the global
-- `Renderer` set by SetEngine so call sites don't change.
function onDef_RenderState(t, mt)
    t.PushAllDefaults = function()
        libphx.RenderState_PushAllDefaults(Renderer)
    end

    t.PushBlendMode = function(value)
        libphx.RenderState_PushBlendMode(Renderer, value)
    end

    t.PushCullFace = function(value)
        libphx.RenderState_PushCullFace(Renderer, value)
    end

    t.PushDepthTest = function(value)
        libphx.RenderState_PushDepthTest(Renderer, value)
    end

    t.PushDepthWritable = function(value)
        libphx.RenderState_PushDepthWritable(Renderer, value)
    end

    t.PushWireframe = function(value)
        libphx.RenderState_PushWireframe(Renderer, value)
    end

    t.PopAll = function()
        libphx.RenderState_PopAll(Renderer)
    end

    t.PopBlendMode = function()
        libphx.RenderState_PopBlendMode(Renderer)
    end

    t.PopWireframe = function()
        libphx.RenderState_PopWireframe(Renderer)
    end

    t.PopDepthTest = function()
        libphx.RenderState_PopDepthTest(Renderer)
    end

    t.PopCullFace = function()
        libphx.RenderState_PopCullFace(Renderer)
    end

    t.PopDepthWritable = function()
        libphx.RenderState_PopDepthWritable(Renderer)
    end
end
