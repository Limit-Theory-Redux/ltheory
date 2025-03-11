local libphx = require('libphx').lib

function onDef_RNG_t(t, mt)
    mt.__index.choose    = function(self, table) return table[libphx.RNG_GetInt(self, 1, #table)] end
    mt.__index.getAxis2  = function(self)
        local out = Vec2f()
        libphx.RNG_GetAxis2(self, out)
        return out
    end
    mt.__index.getAxis3  = function(self)
        local out = Vec3f()
        libphx.RNG_GetAxis3(self, out)
        return out
    end
    mt.__index.getDir2   = function(self)
        local out = Vec2f()
        libphx.RNG_GetDir2(self, out)
        return out
    end
    mt.__index.getDir3   = function(self)
        local out = Vec3f()
        libphx.RNG_GetDir3(self, out)
        return out
    end
    mt.__index.getDisc   = function(self)
        local out = Vec2f()
        libphx.RNG_GetDisc(self, out)
        return out
    end
    mt.__index.getSphere = function(self)
        local out = Vec3f()
        libphx.RNG_GetSphere(self, out)
        return out
    end
    mt.__index.getVec2   = function(self, l, u)
        local out = Vec2f()
        libphx.RNG_GetVec2(self, l, u, out)
        return out
    end
    mt.__index.getVec3   = function(self, l, u)
        local out = Vec3f()
        libphx.RNG_GetVec3(self, l, u, out)
        return out
    end
    mt.__index.getVec4   = function(self, l, u)
        local out = Vec4f()
        libphx.RNG_GetVec4(self, l, u, out)
        return out
    end
    mt.__index.getQuat   = function(self)
        local out = Quat()
        libphx.RNG_GetQuat(self, out)
        return out
    end
end
