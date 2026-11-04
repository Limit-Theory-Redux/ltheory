local libphx = require('libphx').lib

-- `Pipeline.Get(desc)` creates (or finds) the pipeline on the current
-- Renderer and returns its PipelineId; inject the global `Renderer` set by
-- SetEngine so call sites don't pass it.
function onDef_Pipeline(t, mt)
    t.Get = function(desc)
        return libphx.Pipeline_Get(Renderer, desc)
    end
end
