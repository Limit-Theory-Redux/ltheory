local libphx = require('libphx').lib

function onDef_Engine_t(t, mt)
    mt.__index.getVersion = function(...)
        return ffi.string(...)
    end
end
