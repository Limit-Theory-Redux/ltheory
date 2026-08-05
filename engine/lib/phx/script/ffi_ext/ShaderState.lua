local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_ShaderState(t, mt)
    t.FromShaderLoad = function(vsName, fsName)
        local _instance = libphx.ShaderState_FromShaderLoad(Renderer, vsName, fsName)
        return Core.ManagedObject(_instance, libphx.ShaderState_Free)
    end
end

function onDef_ShaderState_t(t, mt)
    mt.__index.setFloat = function(self, name, x)
        libphx.ShaderState_SetFloat(self, Renderer, name, x)
    end

    mt.__index.setFloat2 = function(self, name, x, y)
        libphx.ShaderState_SetFloat2(self, Renderer, name, x, y)
    end

    mt.__index.setFloat3 = function(self, name, x, y, z)
        libphx.ShaderState_SetFloat3(self, Renderer, name, x, y, z)
    end

    mt.__index.setFloat4 = function(self, name, x, y, z, w)
        libphx.ShaderState_SetFloat4(self, Renderer, name, x, y, z, w)
    end

    mt.__index.setInt = function(self, name, x)
        libphx.ShaderState_SetInt(self, Renderer, name, x)
    end

    mt.__index.setInt2 = function(self, name, x, y)
        libphx.ShaderState_SetInt2(self, Renderer, name, x, y)
    end

    mt.__index.setInt3 = function(self, name, x, y, z)
        libphx.ShaderState_SetInt3(self, Renderer, name, x, y, z)
    end

    mt.__index.setInt4 = function(self, name, x, y, z, w)
        libphx.ShaderState_SetInt4(self, Renderer, name, x, y, z, w)
    end

    mt.__index.setMatrix = function(self, name, m)
        libphx.ShaderState_SetMatrix(self, Renderer, name, m)
    end

    mt.__index.setTex1D = function(self, name, t2)
        libphx.ShaderState_SetTex1D(self, Renderer, name, t2)
    end

    mt.__index.setTex2D = function(self, name, t2)
        libphx.ShaderState_SetTex2D(self, Renderer, name, t2)
    end

    mt.__index.setTex3D = function(self, name, t2)
        libphx.ShaderState_SetTex3D(self, Renderer, name, t2)
    end

    mt.__index.setTexCube = function(self, name, t2)
        libphx.ShaderState_SetTexCube(self, Renderer, name, t2)
    end

    mt.__index.start = function(self)
        libphx.ShaderState_Start(self, Renderer)
    end

    mt.__index.stop = function(self)
        libphx.ShaderState_Stop(self, Renderer)
    end
end
