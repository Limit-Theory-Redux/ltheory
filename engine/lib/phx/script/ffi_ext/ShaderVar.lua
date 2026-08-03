local libphx = require('libphx').lib

-- ShaderVar's C functions all take the current Renderer as their first
-- argument now (see ai/multithreaded_rendering.md); inject the global
-- `Renderer` set by SetEngine so call sites don't change.
function onDef_ShaderVar(t, mt)
    t.PushFloat = function(name, x)
        libphx.ShaderVar_PushFloat(Renderer, name, x)
    end

    t.PushFloat2 = function(name, x, y)
        libphx.ShaderVar_PushFloat2(Renderer, name, x, y)
    end

    t.PushFloat3 = function(name, x, y, z)
        libphx.ShaderVar_PushFloat3(Renderer, name, x, y, z)
    end

    t.PushFloat4 = function(name, x, y, z, w)
        libphx.ShaderVar_PushFloat4(Renderer, name, x, y, z, w)
    end

    t.PushInt = function(name, x)
        libphx.ShaderVar_PushInt(Renderer, name, x)
    end

    t.PushInt2 = function(name, x, y)
        libphx.ShaderVar_PushInt2(Renderer, name, x, y)
    end

    t.PushInt3 = function(name, x, y, z)
        libphx.ShaderVar_PushInt3(Renderer, name, x, y, z)
    end

    t.PushInt4 = function(name, x, y, z, w)
        libphx.ShaderVar_PushInt4(Renderer, name, x, y, z, w)
    end

    t.PushMatrix = function(name, m)
        libphx.ShaderVar_PushMatrix(Renderer, name, m)
    end

    t.PushTex1D = function(name, tex)
        libphx.ShaderVar_PushTex1D(Renderer, name, tex)
    end

    t.PushTex2D = function(name, tex)
        libphx.ShaderVar_PushTex2D(Renderer, name, tex)
    end

    t.PushTex3D = function(name, tex)
        libphx.ShaderVar_PushTex3D(Renderer, name, tex)
    end

    t.PushTexCube = function(name, tex)
        libphx.ShaderVar_PushTexCube(Renderer, name, tex)
    end

    t.Pop = function(name)
        libphx.ShaderVar_Pop(Renderer, name)
    end
end
