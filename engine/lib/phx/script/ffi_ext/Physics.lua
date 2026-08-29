local libphx = require('libphx').lib

function onDef_Physics_t(t, mt)
    mt.__index.rayCast    = function(self, ray)
        local r = RayCastResult()
        libphx.Physics_RayCast(self, ray, r)
        return r
    end
    mt.__index.sphereCast = function(self, sphere)
        local r = ShapeCastResult()
        libphx.Physics_SphereCast(self, sphere, r)
        return r
    end
    mt.__index.boxCast    = function(self, pos, rot, halfExtents)
        local r = ShapeCastResult()
        libphx.Physics_BoxCast(self, pos, rot, halfExtents, r)
        return r
    end

    -- These now take the current Renderer as an explicit argument (see
    -- doc/engine/render-thread.md); inject the global `Renderer` set by
    -- SetEngine so call sites don't change.
    mt.__index.drawWireframes = function(self, shader, eye)
        libphx.Physics_DrawWireframes(self, Renderer, shader, eye)
    end
    mt.__index.drawWireframesInRange = function(self, shader, eye, maxRange)
        libphx.Physics_DrawWireframesInRange(self, Renderer, shader, eye, maxRange)
    end
end
