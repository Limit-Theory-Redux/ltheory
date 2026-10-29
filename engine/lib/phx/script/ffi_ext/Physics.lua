local libphx = require('libphx').lib

-- Collision mask with every group: the default for the queries that take a mask.
local ALL_GROUPS = 0xFFFFFFFF

function onDef_Physics_t(t, mt)
    mt.__index.rayCast    = function(self, ray)
        local r = RayCastResult()
        libphx.Physics_RayCast(self, ray, r)
        return r
    end
    -- `exclude` (a RigidBody) and `mask` are optional.
    mt.__index.rayCastFiltered = function(self, ray, exclude, mask)
        local r = RayCastResult()
        libphx.Physics_RayCastFiltered(self, ray, exclude, mask or ALL_GROUPS, r)
        return r
    end
    mt.__index.sphereCast = function(self, pos, radius)
        local r = ShapeCastResult()
        libphx.Physics_SphereCast(self, pos, radius, r)
        return r
    end
    mt.__index.boxCast    = function(self, pos, rot, halfExtents)
        local r = ShapeCastResult()
        libphx.Physics_BoxCast(self, pos, rot, halfExtents, r)
        return r
    end
    -- `exclude` (a RigidBody) and `mask` are optional.
    mt.__index.sphereSweep = function(self, from, to, radius, exclude, mask)
        local r = SphereSweepResult()
        libphx.Physics_SphereSweep(self, from, to, radius, exclude, mask or ALL_GROUPS, r)
        return r
    end
    -- `exclude` (a RigidBody) and `mask` are optional. The result's arrays are valid until
    -- the next `queryRadius`.
    mt.__index.queryRadius = function(self, pos, radius, exclude, mask)
        local r = RadiusQueryResult()
        libphx.Physics_QueryRadius(self, pos, radius, exclude, mask or ALL_GROUPS, r)
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
