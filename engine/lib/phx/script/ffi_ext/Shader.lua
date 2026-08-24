local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_Shader(t, mt)
    t.Create = function(vs, fs)
        local _instance = libphx.Shader_Create(Renderer, vs, fs)
        return Core.ManagedObject(_instance, libphx.Shader_Free)
    end

    t.Load = function(vsName, fsName)
        local _instance = libphx.Shader_Load(Renderer, vsName, fsName)
        return Core.ManagedObject(_instance, libphx.Shader_Free)
    end
end

function onDef_Shader_t(t, mt)
    mt.__index.reload = function(self)
        return libphx.Shader_Reload(self, Renderer)
    end

    mt.__index.getVariable = function(self, name)
        return libphx.Shader_GetVariable(self, Renderer, name)
    end

    mt.__index.hasVariable = function(self, name)
        return libphx.Shader_HasVariable(self, Renderer, name)
    end

    mt.__index.setFloat = function(self, name, value)
        libphx.Shader_SetFloat(self, Renderer, name, value)
    end

    mt.__index.iSetFloat = function(self, index, value)
        libphx.Shader_ISetFloat(self, Renderer, index, value)
    end

    mt.__index.setFloat2 = function(self, name, x, y)
        libphx.Shader_SetFloat2(self, Renderer, name, x, y)
    end

    mt.__index.iSetFloat2 = function(self, index, x, y)
        libphx.Shader_ISetFloat2(self, Renderer, index, x, y)
    end

    mt.__index.setFloat3 = function(self, name, x, y, z)
        libphx.Shader_SetFloat3(self, Renderer, name, x, y, z)
    end

    mt.__index.iSetFloat3 = function(self, index, x, y, z)
        libphx.Shader_ISetFloat3(self, Renderer, index, x, y, z)
    end

    mt.__index.setFloat4 = function(self, name, x, y, z, w)
        libphx.Shader_SetFloat4(self, Renderer, name, x, y, z, w)
    end

    mt.__index.iSetFloat4 = function(self, index, x, y, z, w)
        libphx.Shader_ISetFloat4(self, Renderer, index, x, y, z, w)
    end

    mt.__index.setInt = function(self, name, value)
        libphx.Shader_SetInt(self, Renderer, name, value)
    end

    mt.__index.iSetInt = function(self, index, value)
        libphx.Shader_ISetInt(self, Renderer, index, value)
    end

    mt.__index.setInt2 = function(self, name, x, y)
        libphx.Shader_SetInt2(self, Renderer, name, x, y)
    end

    mt.__index.iSetInt2 = function(self, index, x, y)
        libphx.Shader_ISetInt2(self, Renderer, index, x, y)
    end

    mt.__index.setInt3 = function(self, name, x, y, z)
        libphx.Shader_SetInt3(self, Renderer, name, x, y, z)
    end

    mt.__index.iSetInt3 = function(self, index, x, y, z)
        libphx.Shader_ISetInt3(self, Renderer, index, x, y, z)
    end

    mt.__index.setInt4 = function(self, name, x, y, z, w)
        libphx.Shader_SetInt4(self, Renderer, name, x, y, z, w)
    end

    mt.__index.iSetInt4 = function(self, index, x, y, z, w)
        libphx.Shader_ISetInt4(self, Renderer, index, x, y, z, w)
    end

    mt.__index.setMatrix = function(self, name, value)
        libphx.Shader_SetMatrix(self, Renderer, name, value)
    end

    mt.__index.iSetMatrix = function(self, index, value)
        libphx.Shader_ISetMatrix(self, Renderer, index, value)
    end

    mt.__index.setMatrixT = function(self, name, value)
        libphx.Shader_SetMatrixT(self, Renderer, name, value)
    end

    mt.__index.iSetMatrixT = function(self, index, value)
        libphx.Shader_ISetMatrixT(self, Renderer, index, value)
    end

    mt.__index.indexSetInstanceUniforms = function(self, worldIndex, worldItIndex, scaleIndex, world, worldIt, scale)
        libphx.Shader_IndexSetInstanceUniforms(self, Renderer, worldIndex, worldItIndex, scaleIndex, world, worldIt, scale)
    end

    mt.__index.setTex1D = function(self, name, value)
        libphx.Shader_SetTex1D(self, Renderer, name, value)
    end

    mt.__index.iSetTex1D = function(self, index, value)
        libphx.Shader_ISetTex1D(self, Renderer, index, value)
    end

    mt.__index.setTex2D = function(self, name, value)
        libphx.Shader_SetTex2D(self, Renderer, name, value)
    end

    mt.__index.iSetTex2D = function(self, index, value)
        libphx.Shader_ISetTex2D(self, Renderer, index, value)
    end

    mt.__index.setTex3D = function(self, name, value)
        libphx.Shader_SetTex3D(self, Renderer, name, value)
    end

    mt.__index.iSetTex3D = function(self, index, value)
        libphx.Shader_ISetTex3D(self, Renderer, index, value)
    end

    mt.__index.setTexCube = function(self, name, value)
        libphx.Shader_SetTexCube(self, Renderer, name, value)
    end

    mt.__index.iSetTexCube = function(self, index, value)
        libphx.Shader_ISetTexCube(self, Renderer, index, value)
    end

    mt.__index.start = function(self)
        libphx.Shader_Start(self, Renderer)
    end

    mt.__index.stop = function(self)
        libphx.Shader_Stop(self, Renderer)
    end
end
