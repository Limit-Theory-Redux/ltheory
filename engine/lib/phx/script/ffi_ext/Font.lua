local libphx = require('libphx').lib

-- Font::load now creates its own text shader eagerly, so it needs the
-- current Renderer too (see ai/multithreaded_rendering.md).
function onDef_Font(t, mt)
    t.Load = function(name, size)
        local _instance = libphx.Font_Load(Renderer, name, size)
        return Core.ManagedObject(_instance, libphx.Font_Free)
    end
end

function onDef_Font_t(t, mt)
    mt.__index.getSize = function(self, text)
        local v = Vec4i()
        libphx.Font_GetSize(self, Renderer, text, v)
        return v
    end

    mt.__index.getSize2 = function(self, text)
        return libphx.Font_GetSize2(self, Renderer, text)
    end

    -- Now takes the current Renderer as an explicit argument (see
    -- ai/multithreaded_rendering.md); inject the global `Renderer` set by
    -- SetEngine so call sites don't change.
    mt.__index.draw = function(self, text, x, y, color)
        libphx.Font_Draw(self, Renderer, text, x, y, color)
    end
end
