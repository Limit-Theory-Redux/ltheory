local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- doc/engine/render-thread.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_BoxTree_t(t, mt)
    mt.__index.draw = function(self, maxDepth)
        libphx.BoxTree_Draw(self, Renderer, maxDepth)
    end
end
