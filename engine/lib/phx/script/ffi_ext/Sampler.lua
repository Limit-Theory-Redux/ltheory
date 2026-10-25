local libphx = require('libphx').lib

-- `Sampler.Get(desc)` creates (or finds) the sampler on the current Renderer
-- and returns its SamplerId (the `Samplers.*` presets are always present);
-- inject the global `Renderer` set by SetEngine.
function onDef_Sampler(t, mt)
    t.Get = function(desc)
        return libphx.Sampler_Get(Renderer, desc)
    end
end
