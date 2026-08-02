local libphx = require('libphx').lib

-- These now take the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
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
end
