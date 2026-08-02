local libphx = require('libphx').lib

function onDef_Mesh_t(t, mt)
    mt.__index.getBound  = function(self)
        local b = Box3f()
        libphx.Mesh_GetBound(self, b)
        return b
    end
    mt.__index.getCenter = function(self)
        local v = Vec3f()
        libphx.Mesh_GetCenter(self, v)
        return v
    end

    -- These now take the current Renderer as an explicit argument (see
    -- ai/multithreaded_rendering.md); inject the global `Renderer` set by
    -- SetEngine so call sites don't change.
    mt.__index.computeAO = function(self, radius)
        libphx.Mesh_ComputeAO(self, Renderer, radius)
    end

    mt.__index.computeOcclusion = function(self, sdf, radius)
        libphx.Mesh_ComputeOcclusion(self, Renderer, sdf, radius)
    end
end
