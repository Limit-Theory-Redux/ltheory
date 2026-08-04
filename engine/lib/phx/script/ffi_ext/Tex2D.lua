local libphx = require('libphx').lib

-- Tex2D is now executor-owned (ResourceId, not a raw GL handle); every
-- GL-touching method takes the current Renderer as an explicit argument
-- (see ai/multithreaded_rendering.md). Inject the global `Renderer` set by
-- SetEngine so call sites don't change.

function onDef_Tex2D(t, mt)
    t.Create = function(sx, sy, format)
        local _instance = libphx.Tex2D_Create(Renderer, sx, sy, format)
        return Core.ManagedObject(_instance, libphx.Tex2D_Free)
    end

    t.Load = function(name)
        local _instance = libphx.Tex2D_Load(Renderer, name)
        return Core.ManagedObject(_instance, libphx.Tex2D_Free)
    end

    t.ScreenCapture = function()
        local _instance = libphx.Tex2D_ScreenCapture(Renderer)
        return Core.ManagedObject(_instance, libphx.Tex2D_Free)
    end
end

function onDef_Tex2D_t(t, mt)
    mt.__index.save = function(self, path)
        libphx.Tex2D_Save(self, Renderer, path)
    end

    mt.__index.pop = function(self)
        libphx.Tex2D_Pop(self, Renderer)
    end

    mt.__index.push = function(self)
        libphx.Tex2D_Push(self, Renderer)
    end

    mt.__index.pushLevel = function(self, level)
        libphx.Tex2D_PushLevel(self, Renderer, level)
    end

    mt.__index.clear = function(self, red, green, blue, alpha)
        libphx.Tex2D_Clear(self, Renderer, red, green, blue, alpha)
    end

    mt.__index.deepClone = function(self)
        local _instance = libphx.Tex2D_DeepClone(self, Renderer)
        return Core.ManagedObject(_instance, libphx.Tex2D_Free)
    end

    mt.__index.genMipmap = function(self)
        libphx.Tex2D_GenMipmap(self, Renderer)
    end

    mt.__index.getDataBytes = function(self, pf, df)
        local _instance = libphx.Tex2D_GetDataBytes(self, Renderer, pf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.setAnisotropy = function(self, factor)
        libphx.Tex2D_SetAnisotropy(self, Renderer, factor)
    end

    mt.__index.setDataBytes = function(self, data, pf, df)
        libphx.Tex2D_SetDataBytes(self, Renderer, data, pf, df)
    end

    mt.__index.setMagFilter = function(self, filter)
        libphx.Tex2D_SetMagFilter(self, Renderer, filter)
    end

    mt.__index.setMinFilter = function(self, filter)
        libphx.Tex2D_SetMinFilter(self, Renderer, filter)
    end

    mt.__index.setMipRange = function(self, minLevel, maxLevel)
        libphx.Tex2D_SetMipRange(self, Renderer, minLevel, maxLevel)
    end

    mt.__index.setTexel = function(self, x, y, red, green, blue, alpha)
        libphx.Tex2D_SetTexel(self, Renderer, x, y, red, green, blue, alpha)
    end

    mt.__index.setWrapMode = function(self, mode)
        libphx.Tex2D_SetWrapMode(self, Renderer, mode)
    end

    mt.__index.sample = function(self, x, y)
        return libphx.Tex2D_Sample(self, Renderer, x, y)
    end
end
