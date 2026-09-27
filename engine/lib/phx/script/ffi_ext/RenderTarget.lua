local libphx = require('libphx').lib

-- RenderTarget's C functions all take the current Renderer as their first
-- argument now (see doc/engine/render-thread.md); inject the global
-- `Renderer` set by SetEngine so call sites don't change.
function onDef_RenderTarget(t, mt)
    t.Push = function(sx, sy)
        libphx.RenderTarget_Push(Renderer, sx, sy)
    end

    t.Pop = function()
        libphx.RenderTarget_Pop(Renderer)
    end

    t.BindTex2D = function(tex)
        libphx.RenderTarget_BindTex2D(Renderer, tex)
    end

    t.BindTex2DLevel = function(tex, level)
        libphx.RenderTarget_BindTex2DLevel(Renderer, tex, level)
    end

    t.BindTex3D = function(tex, layer)
        libphx.RenderTarget_BindTex3D(Renderer, tex, layer)
    end

    t.BindTex3DLevel = function(tex, layer, level)
        libphx.RenderTarget_BindTex3DLevel(Renderer, tex, layer, level)
    end

    t.BindTexCube = function(tex, face)
        libphx.RenderTarget_BindTexCube(Renderer, tex, face)
    end

    t.BindTexCubeLevel = function(tex, face, level)
        libphx.RenderTarget_BindTexCubeLevel(Renderer, tex, face, level)
    end

    t.PushTex2D = function(tex)
        libphx.RenderTarget_PushTex2D(Renderer, tex)
    end

    t.PushTex2DLevel = function(tex, level)
        libphx.RenderTarget_PushTex2DLevel(Renderer, tex, level)
    end

    t.PushTex3D = function(tex, layer)
        libphx.RenderTarget_PushTex3D(Renderer, tex, layer)
    end

    t.PushTex3DLevel = function(tex, layer, level)
        libphx.RenderTarget_PushTex3DLevel(Renderer, tex, layer, level)
    end
end
