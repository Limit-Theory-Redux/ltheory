local libphx = require('libphx').lib

function onDef_Matrix_t(t, mt)
    function mt.__add(a, b)
        return (libphx.Matrix_Sum(a, b))
    end

    function mt.__mul(a, b)
        return (libphx.Matrix_Product(a, b))
    end
end
