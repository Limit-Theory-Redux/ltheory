local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_ShaderState_t(t, mt)
    mt.__index.start = function(self)
        libphx.ShaderState_Start(self, Renderer)
    end
end
