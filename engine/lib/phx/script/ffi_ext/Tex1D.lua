local libphx = require('libphx').lib

-- Tex1D is now executor-owned (ResourceId, not a raw GL handle); every
-- GL-touching method takes the current Renderer as an explicit argument
-- (see doc/engine/render-thread.md). Inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_Tex1D(t, mt)
    t.Create = function(size, format)
        local _instance = libphx.Tex1D_Create(Renderer, size, format)
        return Core.ManagedObject(_instance, libphx.Tex1D_Free)
    end
end

function onDef_Tex1D_t(t, mt)
    mt.__index.genMipmap = function(self)
        libphx.Tex1D_GenMipmap(self, Renderer)
    end

    mt.__index.getDataBytes = function(self, pf, df)
        local _instance = libphx.Tex1D_GetDataBytes(self, Renderer, pf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.setDataBytes = function(self, data, pf, df)
        libphx.Tex1D_SetDataBytes(self, Renderer, data, pf, df)
    end

    mt.__index.setMagFilter = function(self, filter)
        libphx.Tex1D_SetMagFilter(self, Renderer, filter)
    end

    mt.__index.setMinFilter = function(self, filter)
        libphx.Tex1D_SetMinFilter(self, Renderer, filter)
    end

    mt.__index.setTexel = function(self, x, red, green, blue, alpha)
        libphx.Tex1D_SetTexel(self, Renderer, x, red, green, blue, alpha)
    end

    mt.__index.setWrapMode = function(self, mode)
        libphx.Tex1D_SetWrapMode(self, Renderer, mode)
    end
end
