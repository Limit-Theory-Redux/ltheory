local libphx = require('libphx').lib

-- These now take the current Renderer as an explicit argument (see
-- doc/engine/render-thread.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_Tex3D(t, mt)
    t.Create = function(sx, sy, sz, format)
        local _instance = libphx.Tex3D_Create(Renderer, sx, sy, sz, format)
        return Core.ManagedObject(_instance, libphx.Tex3D_Free)
    end
end

function onDef_Tex3D_t(t, mt)
    mt.__index.pop = function(self)
        libphx.Tex3D_Pop(self, Renderer)
    end

    mt.__index.push = function(self, layer)
        libphx.Tex3D_Push(self, Renderer, layer)
    end

    mt.__index.pushLevel = function(self, layer, level)
        libphx.Tex3D_PushLevel(self, Renderer, layer, level)
    end

    mt.__index.genMipmap = function(self)
        libphx.Tex3D_GenMipmap(self, Renderer)
    end

    mt.__index.getDataBytes = function(self, pf, df)
        local _instance = libphx.Tex3D_GetDataBytes(self, Renderer, pf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.setDataBytes = function(self, data, pf, df)
        libphx.Tex3D_SetDataBytes(self, Renderer, data, pf, df)
    end

    mt.__index.setMagFilter = function(self, filter)
        libphx.Tex3D_SetMagFilter(self, Renderer, filter)
    end

    mt.__index.setMinFilter = function(self, filter)
        libphx.Tex3D_SetMinFilter(self, Renderer, filter)
    end

    mt.__index.setWrapMode = function(self, mode)
        libphx.Tex3D_SetWrapMode(self, Renderer, mode)
    end
end
