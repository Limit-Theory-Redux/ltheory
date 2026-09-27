local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- doc/engine/render-thread.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_SDF(t, mt)
    t.FromTex3D = function(tex)
        local _instance = libphx.SDF_FromTex3D(Renderer, tex)
        return Core.ManagedObject(_instance, libphx.SDF_Free)
    end
end
