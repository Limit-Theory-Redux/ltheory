local libphx = require('libphx').lib

function onDef_Quat_t(t, mt)
    function mt.__mul(a, b)
        libphx.Quat_Mul(a, b)
    end

    function mt.__index.getAxisX(q)
        local out = Vec3f()
        libphx.Quat_GetAxisX(q, out)
        return out
    end

    function mt.__index.getAxisY(q)
        local out = Vec3f()
        libphx.Quat_GetAxisY(q, out)
        return out
    end

    function mt.__index.getAxisZ(q)
        local out = Vec3f()
        libphx.Quat_GetAxisZ(q, out)
        return out
    end

    function mt.__index.getRight(q)
        local out = Vec3f()
        libphx.Quat_GetRight(q, out)
        return out
    end

    function mt.__index.getUp(q)
        local out = Vec3f()
        libphx.Quat_GetUp(q, out)
        return out
    end

    function mt.__index.getForward(q)
        local out = Vec3f()
        libphx.Quat_GetForward(q, out)
        return out
    end

    function mt.__index.mulV(q, v)
        local out = Vec3f()
        libphx.Quat_MulV(q, v, out)
        return out
    end
end
