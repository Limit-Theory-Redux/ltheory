local libphx = require('libphx').lib

function onDef_Quat_t(t, mt)
    function mt.__mul(a, b)
        return libphx.Quat_Mul(a, b)
    end
end
