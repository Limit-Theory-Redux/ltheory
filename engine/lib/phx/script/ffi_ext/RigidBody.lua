local libphx = require('libphx').lib

function onDef_RigidBody_t(t, mt)
    ---@class RigidBody
    ---@field getPos fun(self: RigidBody, out: Position|nil): Position
    ---@field getPosLocal fun(self: RigidBody, out: Position|nil): Position
    ---@field getRot fun(self: RigidBody, out: Quat|nil): Quat
    ---@field getRotLocal fun(self: RigidBody, out: Quat|nil): Quat
    ---@field getVelocity fun(self: RigidBody, out: Vec3f|nil): Vec3f
    ---@field getVelocityA fun(self: RigidBody, out: Vec3f|nil): Vec3f

    function mt.__index.getPos(self, out)
        local out = out or Position()
        libphx.RigidBody_GetPos(self, out)
        return out
    end

    function mt.__index.getPosLocal(self, out)
        local out = out or Position()
        libphx.RigidBody_GetPosLocal(self, out)
        return out
    end

    --function mt.__index.setPos(q)
    --    local out = out or Position()
    --    libphx.Quat_GetAxisZ(q, out)
    --    return out
    --end
    --
    --function mt.__index.setPosLocal(q)
    --    local out = out or Position()
    --    libphx.Quat_GetRight(q, out)
    --    return out
    --end

    function mt.__index.getRot(self, out)
        local out = out or Quat()
        libphx.RigidBody_GetRot(self, out)
        return out
    end

    function mt.__index.getRotLocal(self, out)
        local out = out or Quat()
        libphx.RigidBody_GetRotLocal(self, out)
        return out
    end

    function mt.__index.getVelocity(self, out)
        local out = out or Vec3f()
        libphx.RigidBody_GetVelocity(self, out)
        return out
    end

    function mt.__index.getVelocityA(self, out)
        local out = out or Vec3f()
        libphx.RigidBody_GetVelocityA(self, out)
        return out
    end

    --function mt.__index.setRot(q, v)
    --    local out = Vec3f()
    --    libphx.Quat_MulV(q, v, out)
    --    return out
    --end
    --
    --function mt.__index.setRotLocal(q, v)
    --    local out = Vec3f()
    --    libphx.Quat_MulV(q, v, out)
    --    return out
    --end
end
