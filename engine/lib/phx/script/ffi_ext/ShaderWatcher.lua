local libphx = require('libphx').lib

-- ShaderWatcher's C functions all take the current Renderer as their first
-- argument (the watcher state lives on RendererData, not a Rust global);
-- inject the global `Renderer` set by SetEngine so call sites don't change.
function onDef_ShaderWatcher(t, mt)
    t.Init = function()
        return libphx.ShaderWatcher_Init(Renderer)
    end

    t.Shutdown = function()
        libphx.ShaderWatcher_Shutdown(Renderer)
    end

    t.IsActive = function()
        return libphx.ShaderWatcher_IsActive(Renderer)
    end

    t.Register = function(shaderKey, vsPath, fsPath)
        libphx.ShaderWatcher_Register(Renderer, shaderKey, vsPath, fsPath)
    end

    t.Poll = function()
        return libphx.ShaderWatcher_Poll(Renderer)
    end

    t.GetChanged = function(index)
        return libphx.ShaderWatcher_GetChanged(Renderer, index)
    end

    t.ClearChanged = function()
        libphx.ShaderWatcher_ClearChanged(Renderer)
    end
end
