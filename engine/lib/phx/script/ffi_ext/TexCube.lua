local libphx = require('libphx').lib

-- These now take the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_TexCube_t(t, mt)
    mt.__index.clear = function(self, red, green, blue, alpha)
        libphx.TexCube_Clear(self, Renderer, red, green, blue, alpha)
    end

    mt.__index.generate = function(self, state)
        libphx.TexCube_Generate(self, Renderer, state)
    end

    mt.__index.genIRMap = function(self, sampleCount)
        local _instance = libphx.TexCube_GenIRMap(self, Renderer, sampleCount)
        return Core.ManagedObject(_instance, libphx.TexCube_Free)
    end
end
