local libphx = require('libphx').lib

-- `pass:finish()` ends the render pass on the current Renderer (see
-- doc/engine/render-api-v2.md); inject the global `Renderer` set by
-- SetEngine so call sites don't pass it.
function onDef_RenderPass_t(t, mt)
    mt.__index.finish = function(self)
        libphx.RenderPass_Finish(self, Renderer)
    end
end
