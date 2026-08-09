local libphx = require('libphx').lib

-- ShaderError's C functions all take the current Renderer as their first
-- argument (the error queue lives on RendererData, not a Rust global);
-- inject the global `Renderer` set by SetEngine so call sites don't change.
function onDef_ShaderError(t, mt)
    t.GetCount = function()
        return libphx.ShaderError_GetCount(Renderer)
    end

    t.HasNewErrors = function()
        return libphx.ShaderError_HasNewErrors(Renderer)
    end

    t.AcknowledgeErrors = function()
        libphx.ShaderError_AcknowledgeErrors(Renderer)
    end

    t.GetShaderKey = function(index)
        return libphx.ShaderError_GetShaderKey(Renderer, index)
    end

    t.GetErrorType = function(index)
        return libphx.ShaderError_GetErrorType(Renderer, index)
    end

    t.GetMessage = function(index)
        return libphx.ShaderError_GetMessage(Renderer, index)
    end

    t.GetTimestamp = function(index)
        return libphx.ShaderError_GetTimestamp(Renderer, index)
    end

    t.Clear = function()
        libphx.ShaderError_Clear(Renderer)
    end

    t.ClearAt = function(index)
        libphx.ShaderError_ClearAt(Renderer, index)
    end

    t.ClearForShader = function(shaderKey)
        libphx.ShaderError_ClearForShader(Renderer, shaderKey)
    end

    t.Update = function()
        libphx.ShaderError_Update(Renderer)
    end

    t.GetLatestMessage = function()
        return libphx.ShaderError_GetLatestMessage(Renderer)
    end

    t.GetLatestShaderKey = function()
        return libphx.ShaderError_GetLatestShaderKey(Renderer)
    end
end
