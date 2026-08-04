local libphx = require('libphx').lib

-- TexCube is now executor-owned (ResourceId, not a raw GL handle); every
-- GL-touching method takes the current Renderer as an explicit argument
-- (see ai/multithreaded_rendering.md). Inject the global `Renderer` set by
-- SetEngine so call sites don't change.

function onDef_TexCube(t, mt)
    t.Create = function(size, format)
        local _instance = libphx.TexCube_Create(Renderer, size, format)
        return Core.ManagedObject(_instance, libphx.TexCube_Free)
    end

    t.Load = function(path)
        local _instance = libphx.TexCube_Load(Renderer, path)
        return Core.ManagedObject(_instance, libphx.TexCube_Free)
    end
end

function onDef_TexCube_t(t, mt)
    mt.__index.clear = function(self, red, green, blue, alpha)
        libphx.TexCube_Clear(self, Renderer, red, green, blue, alpha)
    end

    mt.__index.save = function(self, path)
        libphx.TexCube_Save(self, Renderer, path)
    end

    mt.__index.saveLevel = function(self, path, level)
        libphx.TexCube_SaveLevel(self, Renderer, path, level)
    end

    mt.__index.getDataBytes = function(self, face, level, tf, df)
        local _instance = libphx.TexCube_GetDataBytes(self, Renderer, face, level, tf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.generate = function(self, state)
        libphx.TexCube_Generate(self, Renderer, state)
    end

    mt.__index.genMipmap = function(self)
        libphx.TexCube_GenMipmap(self, Renderer)
    end

    mt.__index.setDataBytes = function(self, data, face, level, tf, df)
        libphx.TexCube_SetDataBytes(self, Renderer, data, face, level, tf, df)
    end

    mt.__index.setMagFilter = function(self, filter)
        libphx.TexCube_SetMagFilter(self, Renderer, filter)
    end

    mt.__index.setMinFilter = function(self, filter)
        libphx.TexCube_SetMinFilter(self, Renderer, filter)
    end

    mt.__index.genIRMap = function(self, sampleCount)
        local _instance = libphx.TexCube_GenIRMap(self, Renderer, sampleCount)
        return Core.ManagedObject(_instance, libphx.TexCube_Free)
    end
end
