local libphx = require('libphx').lib

function onDef_Font_t(t, mt)
    mt.__index.getSize = function(self, text)
        local v = Vec4i()
        libphx.Font_GetSize(self, text, v)
        return v
    end

    -- Now takes the current Renderer as an explicit argument (see
    -- ai/multithreaded_rendering.md); inject the global `Renderer` set by
    -- SetEngine so call sites don't change.
    mt.__index.draw = function(self, text, x, y, color)
        libphx.Font_Draw(self, Renderer, text, x, y, color)
    end
end
