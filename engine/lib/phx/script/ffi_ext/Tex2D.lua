local libphx = require('libphx').lib

-- These now take the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.

function onDef_Tex2D(t, mt)
    t.ScreenCapture = function()
        local _instance = libphx.Tex2D_ScreenCapture(Renderer)
        return Core.ManagedObject(_instance, libphx.Tex2D_Free)
    end
end

function onDef_Tex2D_t(t, mt)
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
end
